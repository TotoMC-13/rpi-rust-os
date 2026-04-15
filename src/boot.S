.section ".text.boot"
.global _start

_start:
    // Leer el ID del procesador
    mrs     x1, mpidr_el1
    and     x1, x1, #3
    // Si no es el Core 0, mandarlo a dormir
    cbz     x1, 2f

1:  // Bucle infinito de bajo consumo para los cores 1, 2 y 3
    wfe
    b       1b

2:  // Configurar el Stack Pointer para Rust (la pila crece hacia abajo desde 0x80000)
    ldr     x1, =_start
    mov     sp, x1

    // Saltar a tu código principal en Rust
    bl      _start_rust
    b       1b // Bucle infinito de seguridad por si Rust retorna (no debería)