use eyre::eyre;
use eyre::Result;
use ihex::Record;
#[derive(Debug, Clone, PartialEq)]
pub struct Patches {
    pub patches: Vec<(u16, [Option<u8>; 256])>,
}
impl Patches {
    pub fn from_path(path: &str) -> Result<Self> {
        let hex = std::fs::read_to_string(path)?;
        Self::from_ihex(&hex)
    }
    pub fn from_ihex(content: &str) -> Result<Self> {
        let mut ret = Patches {
            patches: Vec::new(),
        };
        let mut current_patch: Option<(u16, [Option<u8>; 256])> = None;
        for r in ihex::Reader::new(content) {
            match r? {
                Record::Data { offset, value } => {
                    let mut patch = match current_patch {
                        Some(p) if (p.0..=p.0.saturating_add(255)).contains(&offset) => p,
                        Some(p) => {
                            ret.patches.push(p);
                            (offset, [None; 256])
                        }
                        None => (offset, [None; 256]),
                    };
                    for (i, x) in value.iter().enumerate() {
                        let index = match (i as u16).checked_add(offset) {
                            Some(idx) => idx,
                            None => return Err(eyre!("Patch offset beyond saving")),
                        };
                        if index > patch.0.saturating_add(255) {
                            ret.patches.push(patch);
                            patch = (index, [None; 256]);
                        }
                        patch.1[index as usize - patch.0 as usize] = Some(*x);
                    }
                    current_patch = Some(patch);
                }
                Record::EndOfFile => {
                    if let Some(patch) = current_patch {
                        ret.patches.push(patch)
                    }
                    let addresses: Vec<_> = ret.patches.iter().map(|x| x.0).collect();
                    for (i, x) in addresses.iter().enumerate() {
                        for y in addresses[..i].iter() {
                            if (*x as i32 - *y as i32).abs() < 256 {
                                return Err(eyre!(
                                    "Overlapping memory segments at {} and {}",
                                    x,
                                    y
                                ));
                            }
                        }
                    }
                    return Ok(ret);
                }
                Record::ExtendedSegmentAddress(_) => {
                    return Err(eyre!("Extended Segment Address not supported"))
                }
                Record::StartSegmentAddress { cs: _, ip: _ } => {
                    return Err(eyre!("Start Segment Address not supported"))
                }
                Record::ExtendedLinearAddress(_) => {
                    return Err(eyre!("Extended Linear Address not supported"))
                }
                Record::StartLinearAddress(_) => {
                    return Err(eyre!("Start Linear Address not supported"))
                }
            }
        }
        Err(eyre!("Premature end of ihex"))
    }
    pub fn fill<F>(&self, mut filler: F) -> Result<Vec<(u16, [u8; 256])>>
    where
        F: FnMut(u16) -> Result<u8>,
    {
        let mut ret = Vec::new();
        for (offset, content) in &self.patches {
            let mut new_patch = (*offset, [0; 256]);
            for (i, x) in content.iter().enumerate() {
                let current_offset = offset
                    .checked_add(i as u16)
                    .ok_or(eyre!("Offset out of bounds"))?;
                let byte = match x {
                    None => filler(current_offset)?,
                    Some(b) => *b,
                };
                new_patch.1[i] = byte;
            }
            ret.push(new_patch)
        }
        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn read_ihex() {
        let p = Patches::from_ihex(
            ":08010000E49012349302123462
:07011000901234E0021234EA
:09123400FF7412043081FD142244
:00000001FF
        ",
        )
        .unwrap();
        eprintln!("{:x?}", p);
        let x = p.fill(|n| Ok(n as u8));
        eprintln!("{:x?}", x);
    }
}
