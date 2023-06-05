org 0x7c00 ; address at which the bios will load this executable
bits 16 ; 16 bit mode


    mov ax, 0
    mov ds, ax ; data segment 0
    mov ss, ax ; stack segment 0
    mov es, ax ; extra segment 0?
    mov sp, 0x7c00 ; set stack pointer at the start of this executable

_start:
    mov si, hello

    hlt ; stop here

; si=str, cl=strlen
puts:
    lodsb
    or al, al
    jz .done
    call putc
    jmp puts
.done:
    ret

; al=char
putc:
    mov ah, 0eh
    int 10h
    ret

hello: db 'hello', 0
hello_len: equ $-hello

times 510 - ($ - $$) db 0 ; fill the rest of the sector with 0s
db 0x55, 0xaa ; mark the sector as bootable by setting the bytes 511 and 512
