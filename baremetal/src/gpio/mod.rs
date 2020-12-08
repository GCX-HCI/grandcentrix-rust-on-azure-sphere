/// <summary>Base address of System Control Block, ARM DDI 0403E.b S3.2.2.</summary>
const SCB_BASE: usize = 0xE000ED00;
/// <summary>Base address of NVIC Set-Enable Registers, ARM DDI 0403E.b S3.4.3.</summary>
const NVIC_ISER_BASE: usize = 0xE000E100;
/// <summary>Base address of NVIC Interrupt Priority Registers, ARM DDI 0403E.b S3.4.3.</summary>
const NVIC_IPR_BASE: usize = 0xE000E400;

/// <summary>The IOM4 cores on the MT3620 use three bits to encode interrupt priorities.</summary>
const IRQ_PRIORITY_BITS: u8 = 3;

// Write the supplied 8-bit value to an address formed from the supplied base
// address and offset.
// <param name="baseAddr">Typically the start of a register bank.</param>
// <param name="offset">This value is added to the base address to form the target address.
// It is typically the offset of a register within a bank.</param>
// <param name="value">8-bit value to write to the target address.</param>
#[inline(always)]
fn write_reg8(base_addr: usize, offset: usize, value: u8) {
    //*(volatile uint8_t *)(baseAddr + offset) = value;
    unsafe {
        *((base_addr + offset) as *mut u8) = value;
    }
}

#[inline(always)]
pub fn write_reg32(base_addr: usize, offset: usize, value: u32) {
    //*(volatile uint8_t *)(baseAddr + offset) = value;
    unsafe {
        *((base_addr + offset) as *mut u32) = value;
    }
}

#[inline(always)]
pub fn read_reg32(base_addr: usize, offset: usize) -> u32 {
    //*(volatile uint8_t *)(baseAddr + offset) = value;
    unsafe { *((base_addr + offset) as *mut u32) }
}

#[inline(always)]
pub fn clear_reg32(base_addr: usize, offset: usize, clear_bits: u32) {
    let mut value = read_reg32(base_addr, offset);
    value &= !clear_bits;
    write_reg32(base_addr, offset, value);
}

#[inline(always)]
pub fn set_reg32(base_addr: usize, offset: usize, set_bits: u32) {
    let mut value = read_reg32(base_addr, offset);
    value |= set_bits;
    write_reg32(base_addr, offset, value);
}

#[inline(always)]
pub fn set_nvic_priority(irq_num: u8, pri: u8) {
    write_reg8(
        NVIC_IPR_BASE,
        irq_num as usize,
        pri << (8 - IRQ_PRIORITY_BITS),
    );
}

#[inline(always)]
pub fn enable_nvic_interrupt(irq_num: u8) {
    let offset: usize = 4 * (irq_num as usize / 32);
    let mask = 1u32 << (irq_num % 32);
    set_reg32(NVIC_ISER_BASE, offset, mask);
}

#[inline(always)]
pub fn block_irqs() -> u32 {
    let mut prev_base_pri: u32 = 0;
    let new_base_pri: u32 = 1; // block IRQs priority 1 and above

    unsafe {
        asm!(
            "mrs {prev_base_pri}, BASEPRI",
            prev_base_pri = out(reg) prev_base_pri
        );

        asm!(
            "msr BASEPRI, {new_base_pri}",
            new_base_pri = inout(reg) new_base_pri => _
        );
    }

    return prev_base_pri;
}

#[inline(always)]
pub fn restore_irqs(prev_base_pri: u32) {
    unsafe {
        asm!(
            "msr BASEPRI, {prev_base_pri}",
            prev_base_pri = inout(reg) prev_base_pri => _
        );
    }
}

type GpioBlockType = u16;
// The location of the DIN register depends on the type of block.
pub const GPIO_REG_ADC_DIN: GpioBlockType = 0x04; // PAD GPI Input Data Control Register
pub const GPIO_REG_PWM_DIN: GpioBlockType = 0x04; // GPIO PAD Input Value Register
pub const GPIO_REG_GRP_DIN: GpioBlockType = 0x04; // GPIO PAD Input Value Register
pub const GPIO_REG_ISU_DIN: GpioBlockType = 0x0C; // PAD GPI Input Data Control Register
pub const GPIO_REG_I2SDIN: GpioBlockType = 0x00; // PAD GPI Input Data Control Register

type GpioReg = u16;
const GPIO_REG_DOUT_SET: GpioReg = 0x14; // PAD GPO DATA Output Control Set Register
const GPIO_REG_DOUT_RESET: GpioReg = 0x18; // PAD GPO DATA Output Control Reset Register
const GPIO_REG_OE: GpioReg = 0x20; // PAD GPO Output Enable Control Register
const GPIO_REG_OE_SET: GpioReg = 0x24; // PAD GPO Output Enable Set Control Register
const GPIO_REG_OE_RESET: GpioReg = 0x28; // PAD GPO Output Enable Reset Control Register
const GPIO_REG_IES: GpioReg = 0x60; // PAD IES Control Register
const GPIO_REG_IES_SET: GpioReg = 0x64; // PAD IES SET Control Register
const GPIO_REG_IES_RESET: GpioReg = 0x68; // PAD IES RESET Control Register

// GPIO pins are multiplexed with an ADC block.
pub const GPIO_BLOCK_ADC: u8 = 0;
//  GPIO block also supports PWM.
pub const GPIO_BLOCK_PWM: u8 = 1;
/// <summary>A plain GPIO block.</summary>
pub const GPIO_BLOCK_GRP: u8 = 2;
/// <summary>GPIO pins are multiplexed with I2C / SPI / UART.</summary>
pub const GPIO_BLOCK_ISU: u8 = 3;
/// <summary>GPIO pins are multiplexed with I2S block.</summary>
pub const GPIO_BLOCK_I2S: u8 = 4;

struct BlockType {
    din_reg: u16,
}

static BLOCK_TYPES: [BlockType; 5] = [
    /*GpioBlock_ADC*/ BlockType {
        din_reg: GPIO_REG_ADC_DIN,
    },
    /*GpioBlock_PWM*/ BlockType {
        din_reg: GPIO_REG_PWM_DIN,
    },
    /*GpioBlock_GRP*/ BlockType {
        din_reg: GPIO_REG_GRP_DIN,
    },
    /*GpioBlock_ISU*/ BlockType {
        din_reg: GPIO_REG_ISU_DIN,
    },
    /*GpioBlock_I2S*/ BlockType {
        din_reg: GPIO_REG_I2SDIN,
    },
];

#[derive(Debug, Copy, Clone)]
pub struct GpioBlock {
    /// <summary>The start of the block's register bank.</summary>
    pub base_addr: usize,
    /// <summary>The type of block. This describes how the registers are laid out.</summary>
    pub block_type: GpioBlockType,
    /// <summary>First pin in this block. Each block contains a contiguous range of pins.</summary>
    pub first_pin: u8,
    /// <summary>Number of pins in this block. The first pin is given by <see cref="firstPin" />
    /// and the last pin is firstPin + pinCount - 1.</summary>
    pub pin_count: u8,
}

#[derive(Debug, Copy, Clone)]
struct PinInfo {
    block: GpioBlock,
}

const GPIO_COUNT: usize = 76;
static mut PINS: [Option<PinInfo>; GPIO_COUNT] = [None; GPIO_COUNT];

fn pin_id_to_block(gpio_id: u8, mask: &mut u32) -> Option<GpioBlock> {
    if gpio_id >= GPIO_COUNT as u8 {
        return None;
    }

    let pi1 = unsafe { PINS[gpio_id as usize] };

    match pi1 {
        Some(_) => {}
        None => return None,
    }

    let pi1 = pi1.unwrap();

    let block = pi1.block;

    let idx = gpio_id - block.first_pin;
    *mask = 1u32 << idx;

    return Some(block);
}

fn block_reg_to_ptr32(block: GpioBlock, offset: GpioReg) -> *mut i8 {
    let addr = block.base_addr + offset as usize;
    return addr as *mut i8;
}

fn gpio_write_reg32(block: GpioBlock, reg: GpioReg, value: u32) {
    let ptr = block_reg_to_ptr32(block, reg) as *mut u32;
    unsafe { *ptr = value };
}

fn gpio_read_reg32(block: GpioBlock, reg: GpioReg) -> u32 {
    let ptr = block_reg_to_ptr32(block, reg) as *mut u32;
    unsafe { *ptr }
}

// ---- pin configuration / status ----

fn configure_pin(pin: u8, as_input: bool) -> Result<&'static str, &'static str> {
    let mut pin_mask = 0u32;
    let block = pin_id_to_block(pin, &mut pin_mask);
    match block {
        None => Err(""),
        Some(block) => {
            gpio_write_reg32(block, GPIO_REG_OE_RESET, pin_mask);
            gpio_write_reg32(block, GPIO_REG_IES_RESET, pin_mask);
            if as_input {
                gpio_write_reg32(block, GPIO_REG_IES_SET, pin_mask);
            } else {
                gpio_write_reg32(block, GPIO_REG_OE_SET, pin_mask);
            }

            Ok("")
        }
    }
}

pub fn mt3620_gpio_configure_pin_for_output(pin: u8) -> Result<&'static str, &'static str> {
    return configure_pin(pin, /* asInput */ false);
}

pub fn mt3620_gpio_configure_pin_for_input(pin: u8) -> Result<&'static str, &'static str> {
    return configure_pin(pin, /* asInput */ true);
}

pub fn mt3620_gpio_write(pin: u8, state: bool) -> Result<&'static str, &'static str> {
    let mut pin_mask = 0u32;
    let block = pin_id_to_block(pin, &mut pin_mask);
    match block {
        None => Err(""),
        Some(block) => {
            let reg = if state {
                GPIO_REG_DOUT_SET
            } else {
                GPIO_REG_DOUT_RESET
            };
            gpio_write_reg32(block, reg, pin_mask);
            Ok("")
        }
    }
}

// pub fn Mt3620_Gpio_Read(pin: u8) -> Result<bool, &'static str>
// {
//     let mut pinMask = 0u32;
//     let block = PinIdToBlock(pin, &mut pinMask);
//     match block {
//         None => Err(""),
//         Some(block) => {
//             GpioReg dinReg = blockTypes[pinInfo->block->type].dinReg;
//             let din = Gpio_ReadReg32(block, dinReg);
//             let state = ((din & pinMask) != 0);
//             Ok(state)
//         }
//     }
// }

// ---- initialization ----

pub fn mt3620_gpio_add_block(block: GpioBlock) -> Result<&'static str, &'static str> {
    let low = block.first_pin;
    let high = block.first_pin + block.pin_count - 1;

    if high >= GPIO_COUNT as u8 {
        return Err("INVALID");
    }

    for pin in low..=high {
        match unsafe { PINS[pin as usize] } {
            Some(_) => return Err("EXISTS"),
            None => {}
        }

        unsafe {
            PINS[pin as usize] = Some(PinInfo { block: block });
        }
    }

    Ok("")
}
