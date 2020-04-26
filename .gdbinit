set architecture i386:x86-64
target remote localhost:26000
file target/x86_64-rustos/debug/rustos
b rustos::test::breakpoint
