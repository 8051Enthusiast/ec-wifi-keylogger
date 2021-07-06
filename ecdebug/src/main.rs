use eyre::eyre;
use eyre::Result;
use std::io::{self, Write};
use std::io::{Error, ErrorKind};
use std::thread::sleep;
use std::time::Duration;
use text_io::read;
use x86_64::instructions::port::Port;
mod patch;

const DBG_FIRMWARE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/cmd.bin"));
const FW: &[u8] = include_bytes!("CROM");

fn get_port(portnum: u16) -> Result<Port<u8>> {
    if unsafe { libc::ioperm(portnum as u64, 1, 1) } == -1 {
        return Err(Error::new(
            ErrorKind::PermissionDenied,
            format!("could not get permissions for port {:02x}", portnum),
        )
        .into());
    };
    Ok(Port::<u8>::new(portnum))
}

fn get_single_byte(x: &[u8]) -> Result<u8> {
    x.get(0)
        .copied()
        .ok_or_else(|| Error::new(ErrorKind::UnexpectedEof, "got short read").into())
}

struct Lpc {
    addr: Port<u8>,
    data: Port<u8>,
}

impl Lpc {
    fn new() -> Result<Self> {
        let addr = get_port(0x4e)?;
        let data = get_port(0x4f)?;
        Ok(Lpc { addr, data })
    }
    fn superio_read(&mut self, reg: u8) -> u8 {
        unsafe {
            self.addr.write(reg);
            self.data.read()
        }
    }
    fn superio_write(&mut self, reg: u8, value: u8) {
        unsafe {
            self.addr.write(reg);
            self.data.write(value);
        }
    }
    fn d2_read(&mut self, reg: u8) -> u8 {
        self.superio_write(0x2e, reg);
        self.superio_read(0x2f)
    }
    fn d2_write(&mut self, reg: u8, value: u8) {
        self.superio_write(0x2e, reg);
        self.superio_write(0x2f, value);
    }
    fn i2ec_read(&mut self, addr: u16) -> u8 {
        let [lo, hi] = addr.to_le_bytes();
        self.d2_write(0x11, hi);
        self.d2_write(0x10, lo);
        self.d2_read(0x12)
    }
    fn i2ec_write(&mut self, addr: u16, value: u8) {
        let [lo, hi] = addr.to_le_bytes();
        self.d2_write(0x11, hi);
        self.d2_write(0x10, lo);
        self.d2_write(0x12, value);
    }
    fn reset_patch(&mut self) {
        const REGISTERS: &[u16] = &[0x1049, 0x104c];
        for reg in REGISTERS.iter().rev() {
            self.i2ec_write(reg + 2, 0x03);
            sleep(Duration::from_millis(1))
        }
    }
    fn setup_patch(&mut self, path: &str) -> Result<()> {
        const REGISTERS: &[u16] = &[0x1049, 0x104c];
        self.reset_patch();
        let patches = patch::Patches::from_path(path)?;
        let p = patches.fill(|x| Ok(FW[x as usize]))?;
        if p.len() > REGISTERS.len() {
            return Err(eyre!("Too many patches ({})", p.len()));
        }
        for (offset, _) in p.iter() {
            if *offset > 0xf000 {
                return Err(eyre!("patch offset {} too high", offset));
            }
        }
        eprintln!("{:x?}", p);
        eprintln!("Continue?");
        let response: String = read!();
        if response != "y" {
            return Err(eyre!("User aborted"));
        }
        for ((offset, content), (register, xram_offset)) in
            p.iter().zip(REGISTERS.iter().zip([0xe00u16, 0xf00u16]))
        {
            for (i, x) in content.iter().enumerate() {
                let inner_offset = (*offset as u8).wrapping_add(i as u8);
                self.i2ec_write(xram_offset + inner_offset as u16, *x);
            }
            let [offset_lo, offset_hi] = offset.to_le_bytes();
            self.i2ec_write(*register, offset_lo);
            self.i2ec_write(*register + 1, offset_hi);
            self.i2ec_write(*register + 2, 0x00);
        }
        Ok(())
    }
}

trait Controller {
    fn cmd(&mut self) -> &mut Port<u8>;
    fn data(&mut self) -> &mut Port<u8>;
    fn wait_read_ready(&mut self) -> Result<()> {
        for _ in 0..10000 {
            sleep(Duration::from_micros(10));
            if (unsafe { self.cmd().read() } & 1) == 1 {
                return Ok(());
            }
        }
        Err(Error::new(ErrorKind::TimedOut, "Controller read timed out").into())
    }
    fn wait_write_ready(&mut self) -> Result<()> {
        for _ in 0..10000 {
            sleep(Duration::from_micros(10));
            if (unsafe { self.cmd().read() } & 2) == 0 {
                return Ok(());
            }
        }
        Err(Error::new(ErrorKind::TimedOut, "Controller write timed out").into())
    }
    fn write_cmd(&mut self, cmd: u8) -> Result<()> {
        self.wait_write_ready()?;
        unsafe { self.cmd().write(cmd) };
        Ok(())
    }
    fn write_data(&mut self, data: u8) -> Result<()> {
        self.wait_write_ready()?;
        unsafe { self.data().write(data) };
        Ok(())
    }
    fn read_data(&mut self) -> Result<u8> {
        self.wait_read_ready()?;
        Ok(unsafe { self.data().read() })
    }
    fn send_cmd(&mut self, cmd: &[u8], data: &[u8], ret_bytes: usize) -> Result<Vec<u8>> {
        for c in cmd {
            self.write_cmd(*c)?;
        }
        for d in data {
            self.write_data(*d)?;
        }
        let mut ret = Vec::with_capacity(ret_bytes);
        for _ in 0..ret_bytes {
            ret.push(self.read_data()?);
        }
        Ok(ret)
    }
}

struct Kbc {
    cmd: Port<u8>,
    data: Port<u8>,
}

impl Kbc {
    fn new() -> Result<Self> {
        let mut kbc = Kbc {
            cmd: get_port(0x64)?,
            data: get_port(0x60)?,
        };
        // empty buffer in case there was something left
        loop {
            if (unsafe { kbc.cmd.read() } & 1) == 0 {
                break;
            }
            unsafe { kbc.data.read() };
            sleep(Duration::from_millis(1));
        }
        Ok(kbc)
    }
    fn flash_read(&mut self, addr: u32) -> Result<u8> {
        let [lo, mid, hi, _] = addr.to_le_bytes();
        self.send_cmd(&[0x53], &[hi], 0)?;
        get_single_byte(&self.send_cmd(&[0x53], &[mid, lo], 1)?)
    }
}

impl Controller for Kbc {
    fn cmd(&mut self) -> &mut Port<u8> {
        &mut self.cmd
    }

    fn data(&mut self) -> &mut Port<u8> {
        &mut self.data
    }
}

struct Pm2 {
    cmd: Port<u8>,
    data: Port<u8>,
    flash_mode: bool,
}
impl Pm2 {
    fn new() -> Result<Self> {
        let mut pm2 = Pm2 {
            cmd: get_port(0x6C)?,
            data: get_port(0x68)?,
            flash_mode: false,
        };
        // empty buffer in case there was something left
        loop {
            if (unsafe { pm2.cmd.read() } & 1) == 0 {
                break;
            }
            unsafe { pm2.data.read() };
            sleep(Duration::from_millis(1));
        }
        Ok(pm2)
    }
    fn is_flash(&self) -> Result<()> {
        if !self.flash_mode {
            return Err(
                Error::new(ErrorKind::NotConnected, "controller is not in flash mode").into(),
            );
        }
        Ok(())
    }
    #[allow(dead_code)]
    fn is_not_flash(&self) -> Result<()> {
        if self.flash_mode {
            return Err(Error::new(ErrorKind::NotConnected, "controller is in flash mode").into());
        }
        Ok(())
    }
    fn flash_mode(&mut self) -> Result<()> {
        let ret = self.send_cmd(&[0xdc], &[], 1)?;
        if ret == [0xdc] {
            eprintln!("Warning: device already in debug mode");
        } else if ret != [0x33] {
            return Err(Error::new(
                ErrorKind::ConnectionRefused,
                format!(
                    "flash mode did not return 0x33 but {}",
                    ret.get(0).unwrap_or(&0)
                ),
            )
            .into());
        }
        self.flash_mode = true;
        Ok(())
    }
    fn unflash_mode(&mut self) -> Result<()> {
        self.send_cmd(&[0x05], &[], 0)?;
        self.send_cmd(&[0xfd], &[], 0)?;
        self.flash_mode = false;
        Ok(())
    }
    fn spi_cmd_start(&mut self) -> Result<()> {
        self.send_cmd(&[0x01], &[], 0)?;
        self.send_cmd(&[0x03, 0xff], &[], 0)?;
        self.send_cmd(&[0x03, 0xff], &[], 0)?;
        Ok(())
    }
    fn spi_cmd_scmd(&mut self, cmd: u8) -> Result<()> {
        self.send_cmd(&[0x02, cmd], &[], 0)?;
        Ok(())
    }
    fn spi_cmd_data(&mut self, data: u8) -> Result<()> {
        self.send_cmd(&[0x03, data], &[], 0)?;
        Ok(())
    }
    fn spi_cmd_read(&mut self) -> Result<u8> {
        get_single_byte(&self.send_cmd(&[0x04], &[], 1)?)
    }
    fn spi_cmd_end(&mut self) -> Result<()> {
        self.send_cmd(&[0x05], &[], 0)?;
        Ok(())
    }
    fn spi_cmd(&mut self, cmd: u8, data: &[u8], ret_bytes: usize) -> Result<Vec<u8>> {
        self.is_flash()?;
        let mut ret = Vec::new();
        self.spi_cmd_start()?;
        self.spi_cmd_scmd(cmd)?;
        for d in data {
            self.spi_cmd_data(*d)?;
        }
        for _ in 0..ret_bytes {
            let r = self.spi_cmd_read()?;
            ret.push(r);
        }
        Ok(ret)
    }
    fn spi_wait(&mut self) -> Result<()> {
        self.spi_cmd(5, &[], 0)?;
        for _ in 0..1000 {
            let r = self.spi_cmd_read()?;
            if r & 1 == 0 {
                self.spi_cmd_end()?;
                return Ok(());
            }
        }
        Err(Error::new(ErrorKind::TimedOut, "timed out waiting for spi ready").into())
    }
    fn flash_read(&mut self, addr: u32, len: usize) -> Result<Vec<u8>> {
        let [lo, mid, hi, _] = addr.to_le_bytes();
        self.spi_wait()?;
        let r = self.spi_cmd(0x0b, &[hi, mid, lo, 0], len);
        self.spi_cmd_end()?;
        r
    }
    // forces an endless loop in the EC causing the laptop to do a hard reset by watchdog in ~10 seconds
    #[allow(dead_code)]
    fn ec_kill(&mut self) -> Result<()> {
        self.is_flash()?;
        self.send_cmd(&[0xfe], &[], 0)?;
        Ok(())
    }
}

impl Controller for Pm2 {
    fn cmd(&mut self) -> &mut Port<u8> {
        &mut self.cmd
    }

    fn data(&mut self) -> &mut Port<u8> {
        &mut self.data
    }
}

struct InterfaceCollection {
    kbc: Kbc,
    lpc: Lpc,
    pm2: Pm2,
    enwrite: bool,
}

impl InterfaceCollection {
    fn new() -> Result<Self> {
        let kbc = Kbc::new()?;
        let lpc = Lpc::new()?;
        let pm2 = Pm2::new()?;
        //        let kmod = PMAdapter::new()?;
        Ok(InterfaceCollection {
            kbc,
            lpc,
            pm2,
            //            kmod,
            enwrite: false,
        })
    }
    fn debug_mode(&mut self) -> Result<DebugInterface<'_>> {
        self.pm2.is_flash()?;
        for (i, &x) in DBG_FIRMWARE.iter().enumerate() {
            self.lpc.i2ec_write(0x0600 + i as u16, x);
        }
        self.lpc.i2ec_write(0x07bc, 0x3b); // sjmp 0xfff8
        self.lpc.i2ec_write(0x07f8, 0x02); // ljmp 0xfe00
        self.lpc.i2ec_write(0x07f9, 0xfe);
        self.lpc.i2ec_write(0x07fa, 0x00);
        sleep(Duration::from_millis(10)); // sleep some for good measure
        let x = get_single_byte(&self.pm2.send_cmd(&[0xfc], &[], 1)?)?;
        if x != 0x22 {
            return Err(Error::new(
                ErrorKind::ConnectionRefused,
                format!("flash mode did not return 0x22 but {}", x),
            )
            .into());
        }
        Ok(DebugInterface(self))
    }
}

struct DebugInterface<'a>(&'a mut InterfaceCollection);

impl<'a> DebugInterface<'a> {
    fn cmd(&mut self, cmd: u8, data: [u8; 4]) -> Result<u8> {
        get_single_byte(&self.0.pm2.send_cmd(&[cmd], &data[..], 1)?)
    }
    fn echo_r4(&mut self, val: u8) -> Result<u8> {
        self.cmd(0x03, [val, 0, 0, 0])
    }
    fn echo_r3(&mut self, val: u8) -> Result<u8> {
        self.cmd(0x04, [0, val, 0, 0])
    }
    fn echo_r2(&mut self, val: u8) -> Result<u8> {
        self.cmd(0x05, [0, 0, val, 0])
    }
    fn echo_r1(&mut self, val: u8) -> Result<u8> {
        self.cmd(0x06, [0, 0, 0, val])
    }
    fn read_i(&mut self, addr: u8) -> Result<u8> {
        self.cmd(0x07, [addr, 0, 0, 0])
    }
    fn read_c(&mut self, addr: u16) -> Result<u8> {
        let [lo, hi] = addr.to_le_bytes();
        self.cmd(0x00, [hi, lo, 0, 0])
    }
    fn read_x(&mut self, addr: u16) -> Result<u8> {
        let [lo, hi] = addr.to_le_bytes();
        self.cmd(0x01, [hi, lo, 0, 0])
    }
    fn write_x(&mut self, addr: u16, val: u8) -> Result<u8> {
        let [lo, hi] = addr.to_le_bytes();
        self.cmd(0x02, [hi, lo, val, 0])
    }
    fn write_x_masked(&mut self, addr: u16, val: u8, mask: u8) -> Result<u8> {
        let prev = self.read_x(addr)?;
        let new_val = (prev & !mask) | (val & mask);
        self.write_x(addr, new_val)
    }
    fn reset_patch(&mut self) -> Result<()> {
        const REGISTERS: &[u16] = &[0x1049, 0x104c];
        for reg in REGISTERS {
            self.write_x_masked(reg + 2, 0x03, 0xbf)?;
        }
        Ok(())
    }
    fn setup_patch(&mut self, path: &str) -> Result<()> {
        const REGISTERS: &[u16] = &[0x1049, 0x104c];
        self.reset_patch()?;
        let patches = patch::Patches::from_path(path)?;
        let p = patches.fill(|x| self.read_c(x))?;
        if p.len() > REGISTERS.len() {
            return Err(eyre!("Too many patches ({})", p.len()));
        }
        for (offset, _) in p.iter() {
            if *offset > 0x7f00 {
                return Err(eyre!("patch offset {} too high", offset));
            }
        }
        eprintln!("{:x?}", p);
        eprintln!("Continue?");
        let response: String = read!();
        if response != "y" {
            return Err(eyre!("User aborted"));
        }
        for ((offset, content), (register, xram_offset)) in
            p.iter().zip(REGISTERS.iter().zip([0xe00u16, 0xf00u16]))
        {
            for (i, x) in content.iter().enumerate() {
                let inner_offset = (*offset as u8).wrapping_add(i as u8);
                self.write_x(xram_offset + inner_offset as u16, *x)?;
            }
            let [offset_lo, offset_hi] = offset.to_le_bytes();
            self.write_x(*register, offset_lo)?;
            self.write_x(*register + 1, offset_hi)?;
            self.write_x_masked(*register + 2, 0x00, 0xbf)?;
        }
        Ok(())
    }
    fn leave(mut self) -> Result<()> {
        let x = self.cmd(0x09, [0, 0, 0, 0])?;
        if x != 0x33 {
            eprintln!("Warning: should be 0x33, was 0x{:02x}", x);
        }
        Ok(())
    }
}

fn debug_prompt(stdout: &mut io::Stdout, dbg: &mut DebugInterface) -> Result<bool> {
    eprint!("*> ");
    stdout.flush()?;
    let a: char = read!();
    match a {
        '?' => {
            println!("Help:
z [val] - echo [val] four times through registers
c [addr] - read crom address
C [addr] [len] - read range of crom addresses
r [addr] - read xram address
R [addr] [len] - read range of xram addresses
w [addr] [val] - write xram address
i [addr] - read internal ram
p [path] - patch temporarily using ihex file
P - reset SCAR registers");
        }
        'z' => {
            let input: String = read!();
            let val = u8::from_str_radix(&input, 16)?;
            println!("{:02x}", dbg.echo_r4(val)?);
            println!("{:02x}", dbg.echo_r3(val)?);
            println!("{:02x}", dbg.echo_r2(val)?);
            println!("{:02x}", dbg.echo_r1(val)?);
        }
        'c' => {
            let input: String = read!();
            let addr = u16::from_str_radix(&input, 16)?;
            println!("{:02x}", dbg.read_c(addr)?);
        }
        'C' => {
            let input: String = read!();
            let addr = u16::from_str_radix(&input, 16)?;
            let input: String = read!();
            let len = u16::from_str_radix(&input, 16)?;
            print_hex((addr..=(addr + (len - 1))).map(|x| dbg.read_c(x)))?;
        }
        'r' => {
            let input: String = read!();
            let addr = u16::from_str_radix(&input, 16)?;
            println!("{:02x}", dbg.read_x(addr)?);
        }
        'R' => {
            let input: String = read!();
            let addr = u16::from_str_radix(&input, 16)?;
            let input: String = read!();
            let len = u16::from_str_radix(&input, 16)?;
            print_hex((addr..=(addr + (len - 1))).map(|x| dbg.read_x(x)))?;
        }
        'w' => {
            let input: String = read!();
            let addr = u16::from_str_radix(&input, 16)?;
            let input: String = read!();
            let val = u8::from_str_radix(&input, 16)?;
            println!("{:02x}", dbg.write_x(addr, val)?);
        }
        'i' => {
            let input: String = read!();
            let addr = u8::from_str_radix(&input, 16)?;
            println!("{:02x}", dbg.read_i(addr)?);
        }
        'p' => {
            let input: String = read!();
            dbg.setup_patch(&input)?;
        }
        'P' => {
            dbg.reset_patch()?;
        }
        'q' => return Ok(false),
        otherwise => {
            eprintln!("Invalid char: {}", otherwise);
        }
    }
    Ok(true)
}

fn debug_mode(stdout: &mut io::Stdout, ifc: &mut InterfaceCollection) -> Result<()> {
    let mut dbg = ifc.debug_mode()?;
    loop {
        match debug_prompt(stdout, &mut dbg) {
            Ok(true) => (),
            Ok(false) => break,
            Err(e) => eprintln!("{}", e),
        }
    }
    dbg.leave()?;
    Ok(())
}

fn print_hex(buf: impl Iterator<Item = Result<u8>>) -> Result<()> {
    let mut last = 0;
    for (i, b) in buf.enumerate() {
        let x = b?;
        print!("{:02x}", x);
        match i % 32 + 1 {
            32 => println!(),
            8 | 16 | 24 => print!(" "),
            _ => (),
        }
        last = i;
    }
    if (last + 1) % 32 != 0 {
        println!()
    }
    Ok(())
}

fn read_line(stdout: &mut io::Stdout, ifc: &mut InterfaceCollection) -> Result<bool> {
    eprint!("> ");
    stdout.flush()?;
    let a: char = read!();
    match a {
        'q' => return Ok(false),
        '?' => {
            println!("Help:
r [addr] - read single xram byte using i2ec
R [addr] [len] - read range of xram bytes using i2ec
w [addr] [byte] - write xram byte using i2ec
e flash - enable flash mode
e unflash - disbable flash mode
e write - toggle writes
f [addr] - read flash address
F [addr] [len] - read flash range
m [addr] [len] - read flash range
p [path] - apply patch from hex
P - reset patch
t - enter debug mode
y - leave debug mode (for crashes)");
        }
        'r' => {
            let input: String = read!();
            let addr = u16::from_str_radix(&input, 16)?;
            let byte = ifc.lpc.i2ec_read(addr);
            println!("{:02x}", byte);
        }
        'R' => {
            let input: String = read!();
            let start = u16::from_str_radix(&input, 16)?;
            let input: String = read!();
            let len = u16::from_str_radix(&input, 16)?;
            print_hex((start..=(start + (len - 1))).map(|x| Ok(ifc.lpc.i2ec_read(x))))?;
        }
        'w' => {
            let input_addr: String = read!();
            let addr = u16::from_str_radix(&input_addr, 16)?;
            let input_byte: String = read!();
            let byte = u8::from_str_radix(&input_byte, 16)?;
            if !ifc.enwrite {
                eprintln!("Write disabled, write 'e write' to enable!");
                return Ok(true);
            }
            ifc.lpc.i2ec_write(addr, byte);
        }
        'e' => {
            let input: String = read!();
            match input.as_ref() {
                "write" => {
                    ifc.enwrite = !ifc.enwrite;
                    eprintln!("Set write enable to {}", ifc.enwrite);
                }
                "flash" => {
                    ifc.pm2.flash_mode()?;
                    eprintln!("Enabled flash mode");
                }
                "unflash" => {
                    ifc.pm2.unflash_mode()?;
                    eprintln!("Disabled flash mode");
                }
                otherwise => {
                    eprintln!("Unknown command: {}", otherwise)
                }
            }
        }
        'f' => {
            let input: String = read!();
            let addr = u32::from_str_radix(&input, 16)?;
            println!("{:02x}", ifc.kbc.flash_read(addr)?);
        }
        'F' => {
            let input: String = read!();
            let start = u32::from_str_radix(&input, 16)?;
            let input: String = read!();
            let len = u32::from_str_radix(&input, 16)?;
            print_hex((start..=(start + (len - 1))).map(|x| ifc.kbc.flash_read(x)))?;
        }
        'm' => {
            let input: String = read!();
            let addr = u32::from_str_radix(&input, 16)?;
            let input: String = read!();
            let len = usize::from_str_radix(&input, 16)?;
            print_hex(ifc.pm2.flash_read(addr, len)?.into_iter().map(Ok))?;
        }
        't' => {
            debug_mode(stdout, ifc)?;
        }
        'p' => {
            let path: String = read!();
            if !ifc.enwrite {
                return Err(eyre!("Write disabled, write 'e write' to enable!"));
            }
            ifc.lpc.setup_patch(&path)?;
        }
        'P' => {
            if !ifc.enwrite {
                return Err(eyre!("Write disabled, write 'e write' to enable!"));
            }
            ifc.lpc.reset_patch();
        }
        'y' => {
            let intf = DebugInterface(ifc);
            if let Err(e) = intf.leave() {
                eprintln!("{}", e);
            }
            if let Err(e) = ifc.pm2.unflash_mode() {
                eprintln!("{}", e);
            }
        }
        'k' => {
            let ret = ifc.pm2.send_cmd(&[0x41], &[0xa1], 6)?;
            println!("{}", String::from_utf8_lossy(&ret));
        }
        otherwise => {
            eprintln!("Invalid char: {}", otherwise);
        }
    }
    Ok(true)
}

fn main() -> Result<()> {
    assert!(DBG_FIRMWARE.len() <= 256);
    let mut stdout = std::io::stdout();
    let mut ifc = InterfaceCollection::new()?;
    loop {
        match read_line(&mut stdout, &mut ifc) {
            Ok(true) => (),
            Ok(false) => break,
            Err(e) => eprintln!("{}", e),
        }
    }
    ifc.pm2.unflash_mode()
}
