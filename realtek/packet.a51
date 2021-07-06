packet_start:
; hw tx header (first word is len)
	DB 03Dh, 000h, 028h, 08Ch, 000h, 012h, 000h, 000h
	DB 000h, 000h, 000h, 000h, 000h, 001h, 000h, 000h
	DB 000h, 000h, 000h, 000h, 000h, 000h, 000h, 000h
	DB 000h, 000h, 000h, 000h, 000h, 000h, 000h, 000h
	DB 000h, 080h, 000h, 000h, 000h, 000h, 000h, 000h
; 802.11 frame
; type
	DB 008h, 001h
; duration
	DB 000h, 000h
; bssid (MAC of AP)
	DB 000h, 000h, 000h, 000h, 000h, 000h
; source (MAC of laptop)
	DB 000h, 000h, 000h, 000h, 000h, 000h
; destination (MAC of rpi/receiver)
	DB 000h, 000h, 000h, 000h, 000h, 000h
; fragment/sequence number
	DB 000h, 000h
; llc
	DB 0AAh, 0AAh, 003h, 000h, 000h, 000h
; ethertype (ipv4)
	DB 008h, 000h

; ipv4
	DB 045h, 000h
; total len
	DB 000h, 01dh
; identification
	DB 000h, 000h
	DB 040h, 000h, 040h
; udp
	DB 011h
; header check
	DB 0a9h, 07ah
; source
	DB 000h, 000h, 000h, 000h
; dest
	DB 000h, 000h, 000h, 000h
; udp packet
; source port (55943)
	DB 0dah, 087h
; destination port (10002)
	DB 027h, 012h
; length
	DB 000h, 009h
; checksum
	DB 000h, 000h
packet_data:
	DB 000h
packet_end:
