; ---------------------------------------------------------------------------
; MC6809 CPU Integration Test Suite
; Copyright (c) 2026 Martin Åkesson
; Licensed under the Apache License, Version 2.0
;
; Build:
;   asm6809 -B -o asm/mc6809_test.bin asm/mc6809_test.asm
;
; Each test subroutine is called from run_tests.  On success it returns via
; RTS.  On failure it loads B with the test number and jumps to test_fail,
; which writes B to FAIL_REG ($FF01) — the Rust harness detects this write
; and reports the failing test.  After all 25 tests pass, the code writes to
; PASS_REG ($FF00) and the harness returns HaltReason::Pass.
;
; The reset vector at $FFFE-$FFFF is zero-initialised RAM ($0000), so
; cpu.reset() jumps straight to start: without any loader assistance.
; All interrupt vectors are installed by the startup code at runtime.
;
; Memory layout
; -------------
;   $0000         startup + jmp run_tests
;   $0050         swi_handler / swi2_handler / swi3_handler
;   $0080         test_fail stub
;   $0090         irq_handler / firq_handler / nmi_handler
;   $1000-$101F   scratch RAM (load/store tests)
;   $1FF0         SWI_COUNT  (zero-filled in flat binary)
;   $1FF1         SWI2_COUNT
;   $1FF2         SWI3_COUNT
;   $1FF3         SCRATCH    (single-byte temp for SWI counter compare)
;   $1FF4         IRQ_CALLED (set to 1 by irq_handler)
;   $1FF5         FIRQ_CALLED (set to 1 by firq_handler)
;   $1FF6         NMI_CALLED  (set to 1 by nmi_handler)
;   $2000+        run_tests + t01 .. t28
;   $F000         system stack top  (S grows downward)
;   $F100         user   stack top  (U grows downward)
;   $FF00         PASS_REG  (write any value → pass)
;   $FF01         FAIL_REG  (write test# → fail)
;   $FF02         TRIGGER_IRQ  (write 1 to assert, 0 to deassert)
;   $FF03         TRIGGER_FIRQ (write 1 to assert, 0 to deassert)
;   $FF04         TRIGGER_NMI  (write any value → one NMI pulse)
; ---------------------------------------------------------------------------

PASS_REG    equ  $FF00
FAIL_REG    equ  $FF01
TRIGGER_IRQ  equ  $FF02
TRIGGER_FIRQ equ  $FF03
TRIGGER_NMI  equ  $FF04

S_STACK     equ  $F000
U_STACK     equ  $F100

SWI_COUNT   equ  $1FF0
SWI2_COUNT  equ  $1FF1
SWI3_COUNT  equ  $1FF2
SCRATCH     equ  $1FF3
IRQ_CALLED  equ  $1FF4
FIRQ_CALLED equ  $1FF5
NMI_CALLED  equ  $1FF6

VEC_NMI     equ  $FFFC
VEC_IRQ     equ  $FFF8
VEC_FIRQ    equ  $FFF6
VEC_SWI3    equ  $FFF2
VEC_SWI2    equ  $FFF4
VEC_SWI     equ  $FFFA

; ---------------------------------------------------------------------------
; Startup
; ---------------------------------------------------------------------------

            org  $0000

start:
            lds  #S_STACK
            ldu  #U_STACK

            ; Install interrupt vectors into RAM
            ldx  #swi_handler
            stx  >VEC_SWI
            ldx  #swi2_handler
            stx  >VEC_SWI2
            ldx  #swi3_handler
            stx  >VEC_SWI3
            ldx  #irq_handler
            stx  >VEC_IRQ
            ldx  #firq_handler
            stx  >VEC_FIRQ
            ldx  #nmi_handler
            stx  >VEC_NMI

            ; Clear SWI call counters, scratch byte and interrupt flags
            clr  >SWI_COUNT
            clr  >SWI2_COUNT
            clr  >SWI3_COUNT
            clr  >SCRATCH
            clr  >IRQ_CALLED
            clr  >FIRQ_CALLED
            clr  >NMI_CALLED

            ; Set DP = 0
            clra
            tfr  a,dp
            setdp 0

            jmp  run_tests

; ---------------------------------------------------------------------------
; SWI / SWI2 / SWI3 interrupt handlers
;
; Increment the appropriate call counter, then clobber A/B/X/Y with
; distinctive values so that RTI can be verified to restore them.
; ---------------------------------------------------------------------------

            org  $0050

swi_handler:
            inc  >SWI_COUNT
            lda  #$AA
            ldb  #$BB
            ldx  #$CCDD
            ldy  #$EEFF
            rti

swi2_handler:
            inc  >SWI2_COUNT
            lda  #$AA
            ldb  #$BB
            ldx  #$CCDD
            ldy  #$EEFF
            rti

swi3_handler:
            inc  >SWI3_COUNT
            lda  #$AA
            ldb  #$BB
            ldx  #$CCDD
            ldy  #$EEFF
            rti

; ---------------------------------------------------------------------------
; test_fail: B = failing test number
; Writes B to FAIL_REG (stops the emulator), then loops forever.
; ---------------------------------------------------------------------------

            org  $0080

test_fail:
            stb  >FAIL_REG
            bra  test_fail

; ---------------------------------------------------------------------------
; IRQ / FIRQ / NMI interrupt handlers
;
; irq_handler  – full frame (E=1): clobbers A/B/X/Y to verify RTI restores
;                them; deasserts the IRQ line before RTI so re-entry is safe.
; firq_handler – partial frame (E=0, only CC+PC saved): uses only memory ops
;                so that A/B/X/Y in the interrupted code stay untouched.
; nmi_handler  – full frame, non-maskable: clobbers A/B/X to verify RTI.
; ---------------------------------------------------------------------------

            org  $0090

irq_handler:
            inc  >IRQ_CALLED          ; record call (memory op, no regs)
            lda  #$AA                 ; clobber regs → RTI must restore them
            ldb  #$BB
            ldx  #$CCDD
            ldy  #$EEFF
            clr  >TRIGGER_IRQ         ; deassert IRQ before RTI (CLR: mem write, no regs)
            rti

firq_handler:
            inc  >FIRQ_CALLED         ; record call (memory op, no regs)
            clr  >TRIGGER_FIRQ        ; deassert FIRQ  (memory op, no regs)
            rti                       ; partial pop: restores only CC and PC

nmi_handler:
            inc  >NMI_CALLED          ; record call
            lda  #$AA                 ; clobber regs → RTI must restore them
            ldb  #$BB
            ldx  #$CCDD
            rti

; ---------------------------------------------------------------------------
; run_tests: call every test subroutine, then signal PASS
; ---------------------------------------------------------------------------

            org  $2000

run_tests:
            jsr  t01_imm_loads
            jsr  t02_ext_stores
            jsr  t03_direct
            jsr  t04_indexed
            jsr  t05_alu_add
            jsr  t06_add_flags
            jsr  t07_alu_sub
            jsr  t08_sub_flags
            jsr  t09_logic
            jsr  t10_unary
            jsr  t11_shifts
            jsr  t12_mul_sex_abx
            jsr  t13_cmp
            jsr  t14_andcc_orcc
            jsr  t15_stack_s
            jsr  t16_stack_u
            jsr  t17_tfr
            jsr  t18_exg
            jsr  t19_branches
            jsr  t20_long_branches
            jsr  t21_jsr_rts_bsr
            jsr  t22_swi
            jsr  t23_swi2
            jsr  t24_swi3
            jsr  t25_daa
            jsr  t26_irq
            jsr  t27_firq
            jsr  t28_nmi

            ldb  #28
            stb  >PASS_REG         ; signal all 28 tests passed
            bra  run_tests         ; never reached

; ---------------------------------------------------------------------------
; t01 — Immediate loads: LDA LDB LDD LDX LDY LDU LDS  +  Z/N flags
; ---------------------------------------------------------------------------

t01_imm_loads:
            lda  #$42
            cmpa #$42
            lbne t01_fail

            ; LDA #0 sets Z
            lda  #$00
            lbne t01_fail

            ; LDA #$80 sets N
            lda  #$80
            lbpl t01_fail

            ldb  #$55
            cmpb #$55
            lbne t01_fail

            ldd  #$1234
            cmpd #$1234
            lbne t01_fail

            ldx  #$ABCD
            cmpx #$ABCD
            lbne t01_fail

            ldy  #$CAFE
            cmpy #$CAFE
            lbne t01_fail

            ; LDU — save CC across the U pointer restore
            ldu  #$5678
            cmpu #$5678
            tfr  cc,a
            ldu  #U_STACK
            tfr  a,cc
            lbne t01_fail

            ; LDS — use pshu/pulu to save and restore S
            ; pulu does NOT modify CC, so the CMPS result survives
            pshu s
            lds  #$0800
            cmps #$0800
            pulu s
            lbne t01_fail

            rts

t01_fail:
            ldb  #1
            jmp  test_fail

; ---------------------------------------------------------------------------
; t02 — Extended addressing: STx / LDx store-then-reload roundtrip
; ---------------------------------------------------------------------------

t02_ext_stores:
            lda  #$A5
            sta  >$1000
            clra
            lda  >$1000
            cmpa #$A5
            lbne t02_fail

            ldb  #$5A
            stb  >$1001
            clrb
            ldb  >$1001
            cmpb #$5A
            lbne t02_fail

            ldd  #$1234
            std  >$1002
            ldd  #$0000
            ldd  >$1002
            cmpd #$1234
            lbne t02_fail

            ldx  #$5678
            stx  >$1004
            ldx  #$0000
            ldx  >$1004
            cmpx #$5678
            lbne t02_fail

            ldy  #$9ABC
            sty  >$1006
            ldy  #$0000
            ldy  >$1006
            cmpy #$9ABC
            lbne t02_fail

            ; STU/LDU — save CC across U pointer restore
            ldu  #$DEAD
            stu  >$1008
            ldu  #U_STACK
            ldu  >$1008
            cmpu #$DEAD
            tfr  cc,a
            ldu  #U_STACK
            tfr  a,cc
            lbne t02_fail

            ; STS/LDS — use pshu/pulu (pulu preserves CC)
            pshu s
            lds  #$1800
            sts  >$100A
            lds  >$100A
            cmps #$1800
            pulu s
            lbne t02_fail

            rts

t02_fail:
            ldb  #2
            jmp  test_fail

; ---------------------------------------------------------------------------
; t03 — Direct page addressing  (DP = $10, page at $1000-$10FF)
; ---------------------------------------------------------------------------

t03_direct:
            lda  #$10
            tfr  a,dp
            setdp $10

            lda  #$77
            sta  <$1000
            clra
            lda  <$1000
            cmpa #$77
            lbne t03_fail

            ldb  #$88
            stb  <$1001
            clrb
            ldb  <$1001
            cmpb #$88
            lbne t03_fail

            ldx  #$ABCD
            stx  <$1002
            ldx  #$0000
            ldx  <$1002
            cmpx #$ABCD
            lbne t03_fail

            ; Restore DP before returning
            clra
            tfr  a,dp
            setdp 0
            rts

t03_fail:
            ; Always restore DP even on the fail path
            clra
            tfr  a,dp
            setdp 0
            ldb  #3
            jmp  test_fail

; ---------------------------------------------------------------------------
; t04 — Indexed addressing modes
; ---------------------------------------------------------------------------

t04_indexed:
            ; Prepare two bytes of test data
            lda  #$12
            sta  >$1010
            lda  #$34
            sta  >$1011
            ; Store pointer $1010 at $1012-$1013 for the extended-indirect test
            ldx  #$1010
            stx  >$1012

            ; ,X  zero-offset indexed
            ldx  #$1010
            lda  ,x
            cmpa #$12
            lbne t04_fail

            ; 1,X  constant offset
            lda  1,x
            cmpa #$34
            lbne t04_fail

            ; ,X+  post-increment by 1
            ldx  #$1010
            lda  ,x+
            cmpa #$12
            lbne t04_fail
            cmpx #$1011
            lbne t04_fail

            ; ,X++  post-increment by 2
            ldx  #$1010
            lda  ,x++
            cmpa #$12
            lbne t04_fail
            cmpx #$1012
            lbne t04_fail

            ; ,-X  pre-decrement by 1
            ldx  #$1011
            lda  ,-x
            cmpa #$12
            lbne t04_fail
            cmpx #$1010
            lbne t04_fail

            ; ,--X  pre-decrement by 2
            ldx  #$1012
            lda  ,--x
            cmpa #$12
            lbne t04_fail
            cmpx #$1010
            lbne t04_fail

            ; [>$1012]  extended indirect: reads address from $1012, then data
            lda  [>$1012]
            cmpa #$12
            lbne t04_fail

            ; PCR  PC-relative
            lda  t04_pcr_data,pcr
            cmpa #$99
            lbne t04_fail

            rts

t04_pcr_data:
            fcb  $99

t04_fail:
            ldb  #4
            jmp  test_fail

; ---------------------------------------------------------------------------
; t05 — ALU ADD: ADDA ADDB ADDD ADCA ADCB
; ---------------------------------------------------------------------------

t05_alu_add:
            lda  #$10
            adda #$05
            cmpa #$15
            lbne t05_fail

            ldb  #$20
            addb #$30
            cmpb #$50
            lbne t05_fail

            ldd  #$1000
            addd #$0234
            cmpd #$1234
            lbne t05_fail

            ; ADCA: produce carry via $FF+$01, then add with that carry
            andcc #$FE
            lda  #$FF
            adda #$01              ; result $00, C=1
            lda  #$00
            adca #$00              ; $00 + $00 + C(1) = $01
            cmpa #$01
            lbne t05_fail

            ; ADCB: same pattern
            andcc #$FE
            ldb  #$FF
            addb #$01              ; result $00, C=1
            ldb  #$01
            adcb #$01              ; $01 + $01 + C(1) = $03
            cmpb #$03
            lbne t05_fail

            rts

t05_fail:
            ldb  #5
            jmp  test_fail

; ---------------------------------------------------------------------------
; t06 — ADD flags: Z N C V from ADDA
; ---------------------------------------------------------------------------

t06_add_flags:
            ; Z and C: $FE + $02 = $00, Z=1 C=1
            andcc #$FE
            lda  #$FE
            adda #$02
            lbne t06_fail          ; Z should be set
            bcc  t06_fail          ; C should be set

            ; N and V: $70 + $70 = $E0, N=1 V=1 (positive + positive = negative)
            lda  #$70
            adda #$70
            bpl  t06_fail          ; N should be set
            bvc  t06_fail          ; V should be set

            ; C set: $FF + $01
            andcc #$FE
            lda  #$FF
            adda #$01
            bcc  t06_fail          ; C should be set

            ; C clear: $40 + $3F = $7F
            andcc #$FE
            lda  #$40
            adda #$3F
            bcs  t06_fail          ; C should be clear

            rts

t06_fail:
            ldb  #6
            jmp  test_fail

; ---------------------------------------------------------------------------
; t07 — ALU SUB: SUBA SUBB SUBD SBCA SBCB
; ---------------------------------------------------------------------------

t07_alu_sub:
            lda  #$50
            suba #$10
            cmpa #$40
            lbne t07_fail

            ldb  #$FF
            subb #$0F
            cmpb #$F0
            lbne t07_fail

            ldd  #$1000
            subd #$0100
            cmpd #$0F00
            lbne t07_fail

            ; SBCA: cause borrow, then subtract with that borrow
            andcc #$FE
            lda  #$10
            suba #$20              ; $10 - $20 → borrow, C=1
            lda  #$05
            sbca #$02              ; $05 - $02 - C(1) = $02
            cmpa #$02
            lbne t07_fail

            ; SBCB
            andcc #$FE
            ldb  #$05
            subb #$10              ; borrow, C=1
            ldb  #$20
            sbcb #$05              ; $20 - $05 - C(1) = $1A
            cmpb #$1A
            lbne t07_fail

            rts

t07_fail:
            ldb  #7
            jmp  test_fail

; ---------------------------------------------------------------------------
; t08 — SUB flags: Z N C V from SUBA
; ---------------------------------------------------------------------------

t08_sub_flags:
            ; Z: $10 - $10 = $00
            lda  #$10
            suba #$10
            lbne t08_fail          ; Z should be set

            ; N and C (borrow): $05 - $10 = $F5
            lda  #$05
            suba #$10
            bpl  t08_fail          ; N should be set
            bcc  t08_fail          ; C (borrow) should be set

            ; V: $80 - $01 = $7F  (–128 – 1 overflows to +127)
            lda  #$80
            suba #$01
            bmi  t08_fail          ; N should be clear (result $7F > 0)
            bvc  t08_fail          ; V should be set

            rts

t08_fail:
            ldb  #8
            jmp  test_fail

; ---------------------------------------------------------------------------
; t09 — Logic: ANDA ANDB ORA ORB EORA EORB
; ---------------------------------------------------------------------------

t09_logic:
            lda  #$FF
            anda #$0F
            cmpa #$0F
            lbne t09_fail

            ; AND to zero sets Z
            lda  #$A5
            anda #$5A
            lbne t09_fail          ; $A5 & $5A = $00

            ldb  #$F0
            andb #$3C
            cmpb #$30
            lbne t09_fail

            lda  #$0F
            ora  #$F0
            cmpa #$FF
            lbne t09_fail

            ldb  #$55
            orb  #$AA
            cmpb #$FF
            lbne t09_fail

            lda  #$FF
            eora #$55
            cmpa #$AA
            lbne t09_fail

            ; EOR to zero sets Z
            ldb  #$C3
            eorb #$C3
            lbne t09_fail          ; $C3 ^ $C3 = $00

            ; OR sets N flag
            lda  #$00
            ora  #$80
            bpl  t09_fail          ; N should be set

            rts

t09_fail:
            ldb  #9
            jmp  test_fail

; ---------------------------------------------------------------------------
; t10 — Unary: CLRA CLRB CLR COMA COMB NEGA NEGB INCA INCB DECA DECB TSTA
; ---------------------------------------------------------------------------

t10_unary:
            lda  #$FF
            clra
            lbne t10_fail          ; Z should be set

            ldb  #$AA
            clrb
            lbne t10_fail

            lda  #$55
            sta  >$1000
            clr  >$1000
            lda  >$1000
            lbne t10_fail

            lda  #$55
            coma
            cmpa #$AA
            lbne t10_fail

            ldb  #$F0
            comb
            cmpb #$0F
            lbne t10_fail

            lda  #$01
            nega
            cmpa #$FF
            lbne t10_fail

            ; NEG $80 = $80 (two's-complement wraps, V=1)
            ldb  #$80
            negb
            cmpb #$80
            lbne t10_fail

            lda  #$42
            inca
            cmpa #$43
            lbne t10_fail

            ; INC $FF → $00, Z set
            ldb  #$FF
            incb
            lbne t10_fail

            ; DEC $01 → $00, Z set
            lda  #$01
            deca
            lbne t10_fail

            ; DEC $00 → $FF, N set
            ldb  #$00
            decb
            bpl  t10_fail          ; N should be set

            ; TSTA: sets N for $80, does not change A
            lda  #$80
            tsta
            bpl  t10_fail          ; N should be set

            lda  #$10
            sta  >$1001
            inc  >$1001
            lda  >$1001
            cmpa #$11
            lbne t10_fail

            lda  #$05
            sta  >$1002
            dec  >$1002
            lda  >$1002
            cmpa #$04
            lbne t10_fail

            rts

t10_fail:
            ldb  #10
            jmp  test_fail

; ---------------------------------------------------------------------------
; t11 — Shifts: ASLA ASRA LSRA LSRB ROLA ROLB RORA RORB
; ---------------------------------------------------------------------------

t11_shifts:
            lda  #$41
            asla
            cmpa #$82
            lbne t11_fail

            ; ASLA shifts bit 7 into C
            lda  #$81
            asla
            bcc  t11_fail          ; C should be set (check before cmpa overwrites it)
            cmpa #$02
            lbne t11_fail

            ; LSRA
            lda  #$80
            lsra
            cmpa #$40
            lbne t11_fail

            ; LSRA shifts bit 0 into C
            lda  #$01
            lsra
            lbne t11_fail          ; result $00, Z set
            bcc  t11_fail          ; C should be set

            ; ASRA sign-extends
            lda  #$80
            asra
            cmpa #$C0
            lbne t11_fail

            lda  #$42
            asra
            cmpa #$21
            lbne t11_fail

            ldb  #$FE
            lsrb
            cmpb #$7F
            lbne t11_fail

            ; ROLA: rotate left through C
            andcc #$FE
            lda  #$40
            rola
            bcs  t11_fail          ; C should be clear (check before cmpa overwrites it)
            cmpa #$80
            lbne t11_fail

            orcc  #$01
            lda  #$00
            rola
            cmpa #$01              ; $00 ROL with C=1 → $01
            lbne t11_fail

            ; ROLB: bit 7 shifts into C
            andcc #$FE
            ldb  #$80
            rolb
            bcc  t11_fail          ; C should be set (check before cmpb overwrites it)
            cmpb #$00
            lbne t11_fail

            ; RORA: rotate right through C
            andcc #$FE
            lda  #$01
            rora
            lbne t11_fail          ; $01 ROR C=0 → $00, Z set
            bcc  t11_fail          ; C should be set (bit 0 was 1)

            orcc  #$01
            lda  #$00
            rora
            cmpa #$80              ; $00 ROR C=1 → $80
            lbne t11_fail

            ; RORB
            andcc #$FE
            ldb  #$02
            rorb
            cmpb #$01
            lbne t11_fail

            rts

t11_fail:
            ldb  #11
            jmp  test_fail

; ---------------------------------------------------------------------------
; t12 — MUL SEX ABX
; ---------------------------------------------------------------------------

t12_mul_sex_abx:
            lda  #$0C
            ldb  #$0A
            mul
            cmpd #$0078            ; 12 × 10 = 120 = $78
            lbne t12_fail

            lda  #$FF
            ldb  #$02
            mul
            cmpd #$01FE            ; 255 × 2 = 510 = $01FE
            lbne t12_fail

            ; MUL by zero → D=0
            lda  #$FF
            ldb  #$00
            mul
            lbne t12_fail          ; D should be $0000

            ; SEX: sign-extend B into A
            ldb  #$80
            sex
            cmpd #$FF80
            lbne t12_fail

            ldb  #$7F
            sex
            cmpd #$007F
            lbne t12_fail

            ; ABX: X = X + B (unsigned)
            ldx  #$1000
            ldb  #$10
            abx
            cmpx #$1010
            lbne t12_fail

            rts

t12_fail:
            ldb  #12
            jmp  test_fail

; ---------------------------------------------------------------------------
; t13 — CMP: CMPA CMPB CMPD CMPX CMPY CMPU CMPS
; ---------------------------------------------------------------------------

t13_cmp:
            lda  #$42
            cmpa #$42
            lbne t13_fail

            ; CMPA less (unsigned) → C set
            lda  #$01
            cmpa #$02
            bcc  t13_fail

            ; CMPA greater → C clear
            lda  #$10
            cmpa #$05
            bcs  t13_fail

            ldb  #$80
            cmpb #$80
            lbne t13_fail

            ldd  #$1234
            cmpd #$1234
            lbne t13_fail

            ldx  #$ABCD
            cmpx #$ABCD
            lbne t13_fail

            ldy  #$CAFE
            cmpy #$CAFE
            lbne t13_fail

            ldu  #$1234
            cmpu #$1234
            tfr  cc,a
            ldu  #U_STACK
            tfr  a,cc
            lbne t13_fail

            pshu s
            lds  #$0800
            cmps #$0800
            pulu s
            lbne t13_fail

            rts

t13_fail:
            ldb  #13
            jmp  test_fail

; ---------------------------------------------------------------------------
; t14 — ANDCC / ORCC
; ---------------------------------------------------------------------------

t14_andcc_orcc:
            ; C flag
            andcc #$FE
            orcc  #$01
            bcc   t14_fail         ; C should be set

            andcc #$FE
            bcs   t14_fail         ; C should be clear

            ; N flag (bit 3)
            andcc #$F7
            orcc  #$08
            bpl   t14_fail         ; N should be set

            andcc #$F7
            bmi   t14_fail         ; N should be clear

            ; Z flag (bit 2)
            andcc #$FB
            orcc  #$04
            bne   t14_fail         ; Z should be set

            andcc #$FB
            beq   t14_fail         ; Z should be clear

            ; V flag (bit 1)
            andcc #$FD
            orcc  #$02
            bvc   t14_fail         ; V should be set

            andcc #$FD
            bvs   t14_fail         ; V should be clear

            rts

t14_fail:
            ldb  #14
            jmp  test_fail

; ---------------------------------------------------------------------------
; t15 — PSHS / PULS
; ---------------------------------------------------------------------------

t15_stack_s:
            lda  #$12
            pshs a
            clra
            puls a
            cmpa #$12
            lbne t15_fail

            ldb  #$34
            pshs b
            clrb
            puls b
            cmpb #$34
            lbne t15_fail

            ldx  #$5678
            pshs x
            ldx  #$0000
            puls x
            cmpx #$5678
            lbne t15_fail

            ldy  #$9ABC
            pshs y
            ldy  #$0000
            puls y
            cmpy #$9ABC
            lbne t15_fail

            ; Push and pull multiple registers at once
            lda  #$AA
            ldb  #$BB
            ldx  #$CCDD
            ldy  #$EEFF
            pshs a,b,x,y
            clra
            clrb
            ldx  #$0000
            ldy  #$0000
            puls a,b,x,y
            cmpa #$AA
            lbne t15_fail
            cmpb #$BB
            lbne t15_fail
            cmpx #$CCDD
            lbne t15_fail
            cmpy #$EEFF
            lbne t15_fail

            rts

t15_fail:
            ldb  #15
            jmp  test_fail

; ---------------------------------------------------------------------------
; t16 — PSHU / PULU
; ---------------------------------------------------------------------------

t16_stack_u:
            lda  #$11
            pshu a
            clra
            pulu a
            cmpa #$11
            lbne t16_fail

            ldb  #$22
            pshu b
            clrb
            pulu b
            cmpb #$22
            lbne t16_fail

            ldx  #$3344
            pshu x
            ldx  #$0000
            pulu x
            cmpx #$3344
            lbne t16_fail

            ; Push and pull multiple registers via U stack
            lda  #$AA
            ldb  #$BB
            ldx  #$CCDD
            pshu a,b,x
            clra
            clrb
            ldx  #$0000
            pulu a,b,x
            cmpa #$AA
            lbne t16_fail
            cmpb #$BB
            lbne t16_fail
            cmpx #$CCDD
            lbne t16_fail

            rts

t16_fail:
            ldb  #16
            jmp  test_fail

; ---------------------------------------------------------------------------
; t17 — TFR register transfers
; ---------------------------------------------------------------------------

t17_tfr:
            lda  #$42
            tfr  a,b
            cmpb #$42
            lbne t17_fail

            ldb  #$55
            tfr  b,a
            cmpa #$55
            lbne t17_fail

            ldd  #$1234
            tfr  d,x
            cmpx #$1234
            lbne t17_fail

            ldx  #$ABCD
            tfr  x,d
            cmpd #$ABCD
            lbne t17_fail

            ldx  #$5678
            tfr  x,y
            cmpy #$5678
            lbne t17_fail

            ldy  #$9ABC
            tfr  y,x
            cmpx #$9ABC
            lbne t17_fail

            ; TFR U→X (then restore U)
            ldu  #$CAFE
            tfr  u,x
            cmpx #$CAFE
            tfr  cc,a
            ldu  #U_STACK
            tfr  a,cc
            lbne t17_fail

            ; TFR A→DP, DP→A roundtrip
            lda  #$12
            tfr  a,dp
            setdp $12
            lda  #$00              ; change A so the read-back is meaningful
            tfr  dp,a              ; A should now be $12
            setdp 0
            ldb  #$00
            tfr  b,dp              ; restore DP = 0
            cmpa #$12
            lbne t17_fail

            rts

t17_fail:
            ldb  #17
            jmp  test_fail

; ---------------------------------------------------------------------------
; t18 — EXG register exchange
; ---------------------------------------------------------------------------

t18_exg:
            lda  #$12
            ldb  #$34
            exg  a,b
            cmpa #$34
            lbne t18_fail
            cmpb #$12
            lbne t18_fail

            ldd  #$ABCD
            ldx  #$1234
            exg  d,x
            cmpd #$1234
            lbne t18_fail
            cmpx #$ABCD
            lbne t18_fail

            ldx  #$CAFE
            ldy  #$BABE
            exg  x,y
            cmpx #$BABE
            lbne t18_fail
            cmpy #$CAFE
            lbne t18_fail

            rts

t18_fail:
            ldb  #18
            jmp  test_fail

; ---------------------------------------------------------------------------
; t19 — Short conditional branches
;
; Pattern for each branch B:
;   "should branch":  set condition, B ok_label  /  lbra t19_fail
;   "should not branch": wrong condition, lbXX t19_fail  (long branch, safe range)
; ---------------------------------------------------------------------------

t19_branches:
            ; BEQ
            lda  #$00              ; Z=1
            beq  t19_beq_ok
            lbra t19_fail
t19_beq_ok:
            lda  #$01              ; Z=0
            lbeq t19_fail          ; must NOT branch

            ; BNE
            lda  #$01              ; Z=0
            bne  t19_bne_ok
            lbra t19_fail
t19_bne_ok:
            lda  #$00              ; Z=1
            lbne t19_fail          ; must NOT branch

            ; BCS / BCC
            orcc  #$01             ; C=1
            bcs  t19_bcs_ok
            lbra t19_fail
t19_bcs_ok:
            andcc #$FE             ; C=0
            bcc  t19_bcc_ok
            lbra t19_fail
t19_bcc_ok:
            orcc  #$01             ; C=1
            lbcc  t19_fail         ; must NOT branch
            andcc #$FE             ; C=0
            lbcs  t19_fail         ; must NOT branch

            ; BMI / BPL
            lda  #$80              ; N=1
            bmi  t19_bmi_ok
            lbra t19_fail
t19_bmi_ok:
            lda  #$7F              ; N=0
            bpl  t19_bpl_ok
            lbra t19_fail
t19_bpl_ok:

            ; BVS / BVC
            andcc #$FD
            orcc  #$02             ; V=1
            bvs  t19_bvs_ok
            lbra t19_fail
t19_bvs_ok:
            andcc #$FD             ; V=0
            bvc  t19_bvc_ok
            lbra t19_fail
t19_bvc_ok:

            ; BHI (C=0 and Z=0)
            andcc #$FE
            lda  #$01              ; Z=0
            bhi  t19_bhi_ok
            lbra t19_fail
t19_bhi_ok:

            ; BLS (C=1 or Z=1)
            orcc  #$01             ; C=1
            bls  t19_bls_ok
            lbra t19_fail
t19_bls_ok:

            ; BGE (N=V, both clear here)
            andcc #$F5             ; clear N (bit3) and V (bit1)
            bge  t19_bge_ok
            lbra t19_fail
t19_bge_ok:

            ; BLT (N≠V: N=0, V=1)
            andcc #$F7             ; clear N
            orcc  #$02             ; set V
            blt  t19_blt_ok
            lbra t19_fail
t19_blt_ok:

            ; BGT (Z=0 and N=V, both clear)
            andcc #$F5             ; clear N and V
            lda  #$01              ; Z=0
            bgt  t19_bgt_ok
            lbra t19_fail
t19_bgt_ok:

            ; BLE (Z=1)
            lda  #$00              ; Z=1
            ble  t19_ble_ok
            lbra t19_fail
t19_ble_ok:

            ; BRA (unconditional)
            bra  t19_bra_ok
            lbra t19_fail
t19_bra_ok:

            rts

t19_fail:
            ldb  #19
            jmp  test_fail

; ---------------------------------------------------------------------------
; t20 — Long conditional branches (LBRA, LBEQ, LBNE, LBCS, LBCC, LBGE, LBLT)
; ---------------------------------------------------------------------------

t20_long_branches:
            lbra t20_lbra_ok
            lbra t20_fail
t20_lbra_ok:

            lda  #$00              ; Z=1
            lbeq t20_lbeq_ok
            lbra t20_fail
t20_lbeq_ok:
            lda  #$01              ; Z=0
            lbeq t20_fail          ; must NOT branch

            lda  #$01              ; Z=0
            lbne t20_lbne_ok
            lbra t20_fail
t20_lbne_ok:

            orcc  #$01             ; C=1
            lbcs t20_lbcs_ok
            lbra t20_fail
t20_lbcs_ok:

            andcc #$FE             ; C=0
            lbcc t20_lbcc_ok
            lbra t20_fail
t20_lbcc_ok:

            ; LBGE: N=V=0
            andcc #$F5
            lbge t20_lbge_ok
            lbra t20_fail
t20_lbge_ok:

            ; LBLT: N≠V (N=0, V=1)
            andcc #$F7
            orcc  #$02
            lblt t20_lblt_ok
            lbra t20_fail
t20_lblt_ok:

            rts

t20_fail:
            ldb  #20
            jmp  test_fail

; ---------------------------------------------------------------------------
; t21 — JSR / RTS / BSR / LBSR
; ---------------------------------------------------------------------------

t21_jsr_rts_bsr:
            clra
            jsr  t21_sub
            cmpa #$42
            lbne t21_fail

            clra
            bsr  t21_sub
            cmpa #$42
            lbne t21_fail

            clra
            lbsr t21_sub
            cmpa #$42
            lbne t21_fail

            rts

t21_sub:
            lda  #$42
            rts

t21_fail:
            ldb  #21
            jmp  test_fail

; ---------------------------------------------------------------------------
; t22 — SWI: RTI must restore A B X Y; call counter must increment
; ---------------------------------------------------------------------------

t22_swi:
            ; Compute expected counter value (current + 1)
            lda  >SWI_COUNT
            inca
            sta  >SCRATCH

            ; Load known values then call SWI
            lda  #$01
            ldb  #$02
            ldx  #$0304
            ldy  #$0506
            swi

            ; RTI should have restored all four registers
            cmpa #$01
            lbne t22_fail
            cmpb #$02
            lbne t22_fail
            cmpx #$0304
            lbne t22_fail
            cmpy #$0506
            lbne t22_fail

            ; Verify counter incremented
            lda  >SWI_COUNT
            cmpa >SCRATCH
            lbne t22_fail

            rts

t22_fail:
            ldb  #22
            jmp  test_fail

; ---------------------------------------------------------------------------
; t23 — SWI2: same pattern as t22
; ---------------------------------------------------------------------------

t23_swi2:
            lda  >SWI2_COUNT
            inca
            sta  >SCRATCH

            lda  #$11
            ldb  #$22
            ldx  #$3344
            ldy  #$5566
            swi2

            cmpa #$11
            lbne t23_fail
            cmpb #$22
            lbne t23_fail
            cmpx #$3344
            lbne t23_fail
            cmpy #$5566
            lbne t23_fail

            lda  >SWI2_COUNT
            cmpa >SCRATCH
            lbne t23_fail

            rts

t23_fail:
            ldb  #23
            jmp  test_fail

; ---------------------------------------------------------------------------
; t24 — SWI3: same pattern as t22
; ---------------------------------------------------------------------------

t24_swi3:
            lda  >SWI3_COUNT
            inca
            sta  >SCRATCH

            lda  #$AA
            ldb  #$BB
            ldx  #$CCDD
            ldy  #$EEFF
            swi3

            cmpa #$AA
            lbne t24_fail
            cmpb #$BB
            lbne t24_fail
            cmpx #$CCDD
            lbne t24_fail
            cmpy #$EEFF
            lbne t24_fail

            lda  >SWI3_COUNT
            cmpa >SCRATCH
            lbne t24_fail

            rts

t24_fail:
            ldb  #24
            jmp  test_fail

; ---------------------------------------------------------------------------
; t25 — DAA: decimal-adjust after BCD addition
; ---------------------------------------------------------------------------

t25_daa:
            ; $18 + $19 = $31 binary; H=1 (half carry); DAA → $37
            andcc #$FE
            lda  #$18
            adda #$19
            daa
            cmpa #$37
            lbne t25_fail

            ; $39 + $01 = $3A binary; lower nibble $A > 9; DAA → $40
            andcc #$FE
            lda  #$39
            adda #$01
            daa
            cmpa #$40
            lbne t25_fail

            ; $99 + $01 = $9A; DAA → $00 with carry out
            andcc #$FE
            lda  #$99
            adda #$01
            daa
            lbne t25_fail          ; result should be $00
            bcc  t25_fail          ; carry should be set

            rts

t25_fail:
            ldb  #25
            jmp  test_fail

; ---------------------------------------------------------------------------
; t26 — IRQ: level-triggered, full register frame, RTI restores all regs
;
; Flow:
;   1. Pre-load B/X/Y with known sentinel values.
;   2. Enable IRQ by clearing the I bit (andcc #$EF).
;   3. Write 1 to TRIGGER_IRQ — the harness asserts cpu.set_irq(true) before
;      the very next cpu.step(), so the CPU takes the interrupt before NOP.
;   4. irq_handler increments IRQ_CALLED, clobbers A/B/X/Y, deasserts IRQ,
;      then RTIs — restoring the full frame including B/X/Y.
;   5. Verify IRQ_CALLED == 1 and B/X/Y == original sentinels.
; ---------------------------------------------------------------------------

t26_irq:
            clr  >IRQ_CALLED

            ; Pre-load sentinels in registers that the handler will clobber
            ldb  #$55
            ldx  #$1234
            ldy  #$5678

            andcc #$EF             ; enable IRQ (clear I bit)
            lda  #$01
            sta  >TRIGGER_IRQ      ; assert IRQ; CPU takes it before the next step

            nop                    ; execution resumes here after RTI

            ; handler incremented IRQ_CALLED
            lda  >IRQ_CALLED
            cmpa #$01
            lbne t26_fail

            ; RTI must have restored B, X, Y (handler clobbered them)
            cmpb #$55
            lbne t26_fail
            cmpx #$1234
            lbne t26_fail
            cmpy #$5678
            lbne t26_fail

            orcc #$10              ; re-mask IRQ
            rts

t26_fail:
            orcc #$10              ; re-mask before jumping to test_fail
            ldb  #26
            jmp  test_fail

; ---------------------------------------------------------------------------
; t27 — FIRQ: partial register frame (only CC+PC pushed, E=0 in saved CC)
;
; The handler uses only memory operations (INC, CLR) — no register reads or
; writes — so A, B, X in the interrupted code remain completely untouched.
; After RTI the test verifies they still hold the pre-interrupt sentinels,
; which proves that RTI pulled only 3 bytes (CC + PC) and not 12.
; ---------------------------------------------------------------------------

t27_firq:
            clr  >FIRQ_CALLED

            ; Pre-load sentinels; after FIRQ RTI these must be unchanged
            ldb  #$55
            ldx  #$1234

            andcc #$BF             ; enable FIRQ (clear F bit)
            lda  #$01
            sta  >TRIGGER_FIRQ     ; assert FIRQ; CPU takes it before the next step

            nop                    ; execution resumes here after RTI

            ; A still holds $01 (set just before STA; FIRQ never saves/restores A)
            cmpa #$01
            lbne t27_fail

            ; B and X are unchanged (not saved by FIRQ, not touched by handler)
            cmpb #$55
            lbne t27_fail
            cmpx #$1234
            lbne t27_fail

            ; handler incremented FIRQ_CALLED
            lda  >FIRQ_CALLED
            cmpa #$01
            lbne t27_fail

            orcc #$40              ; re-mask FIRQ
            rts

t27_fail:
            orcc #$40              ; re-mask before jumping to test_fail
            ldb  #27
            jmp  test_fail

; ---------------------------------------------------------------------------
; t28 — NMI: edge-triggered, non-maskable, full register frame
;
; Both the IRQ mask (I) and FIRQ mask (F) are explicitly set before firing the
; NMI to demonstrate that the Non-Maskable Interrupt fires regardless.
; Writing to TRIGGER_NMI delivers a one-shot NMI pulse via cpu.trigger_nmi().
; nmi_handler clobbers A/B/X; RTI must restore the original sentinels.
; ---------------------------------------------------------------------------

t28_nmi:
            clr  >NMI_CALLED

            ; Pre-load sentinels that the handler will clobber
            ldb  #$77
            ldx  #$AABB

            ; Mask both IRQ and FIRQ (I=1, F=1) — NMI must still fire
            orcc #$50

            lda  #$01
            sta  >TRIGGER_NMI      ; fire NMI pulse; CPU takes it before the next step

            nop                    ; execution resumes here after RTI

            ; handler incremented NMI_CALLED
            lda  >NMI_CALLED
            cmpa #$01
            lbne t28_fail

            ; RTI must have restored B and X (handler clobbered them)
            cmpb #$77
            lbne t28_fail
            cmpx #$AABB
            lbne t28_fail

            rts

t28_fail:
            ldb  #28
            jmp  test_fail
