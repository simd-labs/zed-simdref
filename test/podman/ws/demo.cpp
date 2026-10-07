#include <immintrin.h>

int main() {
    asm volatile("vaddps %ymm1, %ymm2, %ymm0");
    int x = 42;
    return x;
}
