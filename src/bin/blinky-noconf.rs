#![no_std]
#![no_main]

use panic_halt as _; // panic handler
use rtic::app;

#[app(device = stm32f4::stm32f401, peripherals = true)]
mod app {
    use cortex_m::peripheral::syst::SystClkSource;
    use cortex_m_rtic_trace::{self, trace};
    use stm32f4xx_hal::stm32;
    use cortex_m::asm;

    #[resources]
    struct Resources {
        GPIOA: stm32::GPIOA,
    }

    #[init]
    fn init(mut ctx: init::Context) -> (init::LateResources, init::Monotonics) {
        let mut syst = ctx.core.SYST;

        // Allow debugger to attach while sleeping (WFI)
        ctx.device.DBGMCU.cr.modify(|_, w| {
            w.dbg_sleep().set_bit();
            w.dbg_standby().set_bit();
            w.dbg_stop().set_bit()
        });

        // configures the system timer to trigger a SysTick exception every second
        syst.set_clock_source(SystClkSource::Core);
        syst.set_reload(16_000_000); // period = 1s
        syst.enable_counter();
        syst.enable_interrupt();

        // power on GPIOA, RM0368 6.3.11
        ctx.device.RCC.ahb1enr.modify(|_, w| w.gpioaen().set_bit());
        // configure PA5 as output, RM0368 8.4.1
        ctx.device.GPIOA.moder.modify(|_, w| w.moder5().bits(1));

        (
            init::LateResources {
                GPIOA: ctx.device.GPIOA,
            },
            init::Monotonics(),
        )
    }

    #[task(binds = SysTick, resources = [GPIOA])]
    fn toggle(mut ctx: toggle::Context) {
        static mut TOGGLE: bool = false;
        if *TOGGLE {
            ctx.resources
                .GPIOA
                .lock(|gpioa| gpioa.bsrr.write(|w| w.bs5().set_bit()));
        } else {
            ctx.resources
                .GPIOA
                .lock(|gpioa| gpioa.bsrr.write(|w| w.br5().set_bit()));
        }
        *TOGGLE = !*TOGGLE;
    }
}
