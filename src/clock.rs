//! SoC clock configuration
// 其实和gpio两个模块同属GLB外设
// 时钟控制器
use crate::pac;
use crate::gpio::ClkCfg;
use core::num::NonZeroU32;
use embedded_time::rate::Hertz;
use crate::pac::Peripherals;
use embedded_hal::blocking::delay::{DelayUs, DelayMs};
use crate::delay::*;
pub struct Clocks {
    uart_clk_div: u8,
}

impl Clocks {
    pub const fn uart_clk(&self) -> Hertz {
        Hertz(160_000_000 / self.uart_clk_div as u32)
    }
}

/// Strict clock configurator
///
/// This configurator only accepts strictly accurate value. If all available frequency
/// values after configurated does not strictly equal to the desired value, the `freeze`
/// function panics. Users must be careful to ensure that the output frequency values
/// can be strictly configurated into using input frequency values and internal clock
/// frequencies.
///
/// If you need to get most precise frequenct possible (other than the stictly accutare
/// value only), use configurator `Precise` instead.
///
/// For example if 49.60MHz and 50.20MHz are able to be configurated prefectly, input
/// 50MHz into `Strict` would result in a panic when performing `freeze`; however input
/// same 50MHz into `Precise` it would not panic, but would set and freeze into
/// 50.20MHz as the frequency error is smallest.
pub struct Strict {
    target_uart_clk: Option<NonZeroU32>,
}

/// HBN root clock type definition
#[repr(u8)]
enum HBN_ROOT_CLK_Type {
    RC32M = 0,           // use RC32M as root clock
    XTAL  = 1,           // use XTAL as root clock
    PLL   = 2,           // use PLL as root clock
}

/**
 *  @brief PLL XTAL type definition
 */
 #[repr(u8)]
enum GLB_PLL_XTAL_Type {
    NONE        = 0,     // XTAL is none
    XTAL_24M    = 1,     // XTAL is 24M
    XTAL_32M    = 2,     // XTAL is 32M
    XTAL_38P4M  = 3,     // XTAL is 38.4M
    XTAL_40M    = 4,     // XTAL is 40M
    XTAL_26M    = 5,     // XTAL is 26M
    RC32M       = 6,     // XTAL is RC32M
}

/// Minimal implementation of power-on pll. Currently hard-coded to 40M xtal
fn pds_power_on_pll(dp: &mut Peripherals, xtal: GLB_PLL_XTAL_Type) {
    let mut delay = McycleDelay::new(dp.HBN.hbn_rsv2.read().bits());

    dp.PDS.pu_rst_clkpll.modify(|_r, w| unsafe {w
        .pu_clkpll_sfreg().set_bit()
    });

    //DelayUs(5);
    delay.try_delay_us(5).unwrap();

    /* pu_clkpll=1 */
    dp.PDS.pu_rst_clkpll.modify(|_r, w| unsafe {w
        .pu_clkpll().set_bit()
    });

    /* clkpll_pu_cp=1 */
    /* clkpll_pu_pfd=1 */
    /* clkpll_pu_fbdv=1 */
    /* clkpll_pu_postdiv=1 */
    dp.PDS.pu_rst_clkpll.modify(|_r, w| unsafe {w
        .clkpll_pu_cp().set_bit()
        .clkpll_pu_pfd().set_bit()
        .clkpll_pu_fbdv().set_bit()
        .clkpll_pu_postdiv().set_bit()
    });
    //DelayUs(5);
    delay.try_delay_us(5).unwrap();

    // /* clkpll_sdm_reset=1 */
    // tmpVal=BL_RD_REG(PDS_BASE,PDS_PU_RST_CLKPLL);
    // tmpVal=BL_SET_REG_BIT(tmpVal,PDS_CLKPLL_SDM_RESET);
    // BL_WR_REG(PDS_BASE,PDS_PU_RST_CLKPLL,tmpVal);
    dp.PDS.pu_rst_clkpll.modify(|_r, w| unsafe {w
        .clkpll_sdm_reset().set_bit()
    });
    // BL602_Delay_US(1);
    delay.try_delay_us(1).unwrap();

    // /* clkpll_reset_fbdv=1 */
    // tmpVal=BL_RD_REG(PDS_BASE,PDS_PU_RST_CLKPLL);
    // tmpVal=BL_SET_REG_BIT(tmpVal,PDS_CLKPLL_RESET_FBDV);
    // BL_WR_REG(PDS_BASE,PDS_PU_RST_CLKPLL,tmpVal);
    dp.PDS.pu_rst_clkpll.modify(|_r, w| unsafe {w
        .clkpll_reset_fbdv().set_bit()
    });
    // BL602_Delay_US(2);
    delay.try_delay_us(2).unwrap();

    // /* clkpll_reset_fbdv=0 */
    // tmpVal=BL_RD_REG(PDS_BASE,PDS_PU_RST_CLKPLL);
    // tmpVal=BL_CLR_REG_BIT(tmpVal,PDS_CLKPLL_RESET_FBDV);
    // BL_WR_REG(PDS_BASE,PDS_PU_RST_CLKPLL,tmpVal);
    dp.PDS.pu_rst_clkpll.modify(|_r, w| unsafe {w
        .clkpll_reset_fbdv().clear_bit()
    });
    // BL602_Delay_US(1);
    delay.try_delay_us(1).unwrap();

    // /* clkpll_sdm_reset=0 */
    // tmpVal=BL_RD_REG(PDS_BASE,PDS_PU_RST_CLKPLL);
    // tmpVal=BL_CLR_REG_BIT(tmpVal,PDS_CLKPLL_SDM_RESET);
    // BL_WR_REG(PDS_BASE,PDS_PU_RST_CLKPLL,tmpVal);
    dp.PDS.pu_rst_clkpll.modify(|_r, w| unsafe {w
        .clkpll_sdm_reset().clear_bit()
    });
}

fn aon_power_on_xtal(dp: &mut Peripherals) {
    dp.AON.rf_top_aon.modify(|_, w| unsafe { w
        .pu_xtal_aon().set_bit()
        .pu_xtal_buf_aon().set_bit()
    });

    let mut delaysrc = McycleDelay::new(dp.HBN.hbn_rsv2.read().bits());
    let mut timeOut:u32 = 0;
    delaysrc.try_delay_us(10).unwrap();
    while dp.AON.tsen.read().xtal_rdy().bit_is_clear() && timeOut < 120{
        delaysrc.try_delay_us(10);
        timeOut+=1;
    }
    // TODO: error out on timeout
}

/// Setup XTAL and PLL for system clock
/// TODO: finish clock init - some parts are hard-coded for 40Mhz XTAL + 160Mhz target clock
fn glb_set_system_clk(dp: &mut Peripherals) {
    /* reg_bclk_en = reg_hclk_en = reg_fclk_en = 1, cannot be zero */
    // tmpVal = BL_SET_REG_BIT(tmpVal,GLB_REG_BCLK_EN);
    // tmpVal = BL_SET_REG_BIT(tmpVal,GLB_REG_HCLK_EN);
    // tmpVal = BL_SET_REG_BIT(tmpVal,GLB_REG_FCLK_EN);
    // BL_WR_REG(GLB_BASE,GLB_CLK_CFG0,tmpVal);
    dp.GLB.clk_cfg0.modify(|_, w| unsafe { w
        .reg_bclk_en().set_bit()
        .reg_hclk_en().set_bit()
        .reg_fclk_en().set_bit()
    });

    //HBN_Set_ROOT_CLK_Sel(HBN_ROOT_CLK_RC32M)
     /* Before config XTAL and PLL ,make sure root clk is from RC32M */
    dp.HBN.hbn_glb.modify(|_,w| unsafe { w
        .hbn_root_clk_sel().bits(0)
    });

    dp.GLB.clk_cfg0.modify(|_,w| unsafe { w
        .reg_hclk_div().bits(0)
        .reg_bclk_div().bits(0)
    });

    // Update sysclock
    dp.HBN.hbn_rsv2.write(|w| unsafe { w
        .bits(32_000_000)
    });

    /* Select PKA clock from hclk */
    dp.GLB.swrst_cfg2.modify(|_,w| unsafe { w
        .pka_clk_sel().clear_bit()
    });

    /* AON_Power_On_XTAL(); */
    aon_power_on_xtal(dp);

    /* always power up PLL and enable all PLL clock output */
    pds_power_on_pll(dp, GLB_PLL_XTAL_Type::XTAL_40M);
    let mut delay = McycleDelay::new(dp.HBN.hbn_rsv2.read().bits());
    delay.try_delay_us(55).unwrap();

    // PDS_Enable_PLL_All_Clks()
    dp.PDS.clkpll_output_en.modify(|r, w| unsafe {w
        .bits(r.bits() | 0x1FF)
    });
    
    /* reg_pll_en = 1, cannot be zero */
    dp.GLB.clk_cfg0.modify(|r, w| unsafe {w
        .reg_pll_en().set_bit()
    });

    /* select pll output clock before select root clock */
    // sets to clkFreq-GLB_SYS_CLK_PLL48M, where PLL160M is 2 more than PLL48M
    dp.GLB.clk_cfg0.modify(|r, w| unsafe {w
        .reg_pll_sel().bits(2)
    });

    /* select root clock */
    // TODO: bring back target clocks other than GLB_SYS_CLK_PLL160M

    // L1C_IROM_2T_Access_Set(ENABLE);
    //dp.L1C_IROM_2T_Access_Set
    dp.L1C.l1c_config.modify(|r, w| unsafe {w
        .irom_2t_access().set_bit()
    });
    // GLB_Set_System_CLK_Div(0,1);

    dp.GLB.clk_cfg0.modify(|_,w| unsafe { w
        .reg_hclk_div().bits(0)
        .reg_bclk_div().bits(1)
    });

    // HBN_Set_ROOT_CLK_Sel(HBN_ROOT_CLK_PLL);
    dp.HBN.hbn_glb.modify(|_,w| unsafe { w
        .hbn_root_clk_sel().bits(0)
    });
    // SystemCoreClockSet(160*1000*1000);
    dp.HBN.hbn_rsv2.write(|w| unsafe { w
        .bits(160_000_000)
    });

    // GLB_CLK_SET_DUMMY_WAIT;
    delay.try_delay_us(500).unwrap();

    // /* select PKA clock from 120M since we power up PLL */
    // NOTE: This isn't documented in the datasheet!
    // GLB_Set_PKA_CLK_Sel(GLB_PKA_CLK_PLL120M);
    dp.GLB.swrst_cfg2.write(|w| unsafe { w
        .pka_clk_sel().set_bit()
    });
}

impl Strict {
    /// Create a strict configurator
    pub fn new() -> Self {
        Strict {
            target_uart_clk: None,
        }
    }

    /// Sets the desired frequency for the UART-CLK clock
    pub fn uart_clk(mut self, freq: impl Into<Hertz>) -> Self {
        let freq_hz = freq.into().0;
        self.target_uart_clk = NonZeroU32::new(freq_hz);
        self
    }

    /// Calculate and balance clock registers to configure into the given clock value.
    /// If accurate value is not possible, this function panics. 
    /// 
    /// Be aware that Rust's panic is sometimes not obvious on embedded devices; if your
    /// program didn't execute as expected, or the `pc` is pointing to somewhere weird
    /// (usually `abort: j abort`), it's likely that this function have panicked. 
    /// Breakpoint on `rust_begin_unwind` may help debugging.
    ///
    /// # Panics
    ///
    /// If strictly accurate value of given `ck_sys` etc. is not reachable, this function
    /// panics. 
    pub fn freeze(self, clk_cfg: &mut ClkCfg) -> Clocks {
        drop(clk_cfg); // logically use its ownership
        let uart_clk = self.target_uart_clk.map(|f| f.get()).unwrap_or(40_000_000);
        let uart_clk_div = {
            let ans = 160_000_000 / uart_clk;
            if !(ans >= 1 && ans <= 7) || ans * uart_clk != 160_000_000 {
                panic!("unreachable uart_clk")
            }
            ans as u8
        };
        let glb = unsafe { &*pac::GLB::ptr() };
        glb.clk_cfg2.write(|w| unsafe { w
            .uart_clk_div().bits(uart_clk_div)
            .uart_clk_en().set_bit()
        });
        Clocks {
            uart_clk_div
        }
    }
}
