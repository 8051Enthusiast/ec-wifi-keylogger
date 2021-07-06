IN	XDATA	01514h
OUT	XDATA	01511h
STATUS	XDATA	01510h
STARTADDR	CODE	0FE00h
ORG STARTADDR
START:
	MOV A, #022h		; send back 022h to the host to say that the routine was entered
PRINT:
	MOV DPTR, #OUT		; send whatever we have returned in A to host
	MOVX @DPTR, A
	MOV A, #0ffh		; set command to 0ff so we don't accidentally execute any command we don't want
ST_REPL:
	MOV R7, A		; (r7 is command,
	MOV R0, #04h		; r4-r1 is data)
REPL:
	MOV DPTR, #STATUS
	MOVX A, @DPTR
	JNB ACC.1, REPL		; wait for byte

	MOV C, ACC.3
	MOV DPTR, #IN
	MOVX A, @DPTR
	JC ST_REPL		; if cmd byte, move cmd byte to r7
				; and prepare to save data bytes in r4-r1

	MOV @R0, A		; else data byte: save it to register pointed to by r0
	DJNZ R0, REPL		; stop when we reach r0 and execute actual command

	MOV A, R7
	CJNE A, #(CMD_TABLE_END - CMD_TABLE), NEXT
NEXT:
	JNC PRINT		; make sure we don't overshoot  (returns cmd byte itself)
	MOV DPTR, #CMD_TABLE
	ACALL EVAL
	SJMP PRINT
EVAL:
	MOVC A, @A+DPTR
	JMP @A+DPTR
ENTRY	MACRO	FUNCTION
	DB (FUNCTION - CMD_TABLE)
ENDM
CMD_TABLE:
	ENTRY READ_C		; 00
	ENTRY READ_X		; 01
	ENTRY WRITE_X		; 02
	ENTRY ECHO_R4		; 03
	ENTRY ECHO_R3		; 04
	ENTRY ECHO_R2		; 05
	ENTRY ECHO_R1		; 06
	ENTRY READ_I		; 07
	ENTRY READ_FLASH	; 08
	ENTRY LEAVE		; 09
CMD_TABLE_END:
READ_C:
	MOV DPH, R4
	MOV DPL, R3
	CLR A
	MOVC A, @A+DPTR
	RET
READ_X:
	MOV DPH, R4
	MOV DPL, R3
	MOVX A, @DPTR
	RET
WRITE_X:
	MOV DPH, R4
	MOV DPL, R3
	MOV A, R2
	MOVX @DPTR, A
	RET
ECHO_R4:
	MOV A, R4
	RET
ECHO_R3:
	MOV A, R3
	RET
ECHO_R2:
	MOV A, R2
	RET
ECHO_R1:
	MOV A, R1
	RET
READ_I:
	MOV A, R4
	MOV R0, A
	MOV A, @R0
	RET
READ_FLASH:
	MOV DPTR, #0103Bh
	MOV A, R4
	MOVX @DPTR, A
	MOV A, R3
	INC DPTR
	MOVX @DPTR, A
	MOV A, R2
	INC DPTR
	MOVX @DPTR, A
	CLR A
	INC DPTR
	MOVX @DPTR, A
	INC DPTR
	MOVX A, @DPTR
	RET
LEAVE:
	LJMP 0ff09h
END
