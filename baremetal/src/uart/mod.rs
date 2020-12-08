use super::gpio::*;

pub enum UartId {
    UartCM4Debug,
    UartIsu0,
    UartIsu1,
    UartIsu2,
    UartIsu3,
    UartIsu4,
}

const UART_PRIORITY: u8 = 2;

const TX_FIFO_DEPTH: u32 = 16;

type EnqCtrType = u16;

const TX_BUFFER_SIZE: usize = 256;
const TX_BUFFER_MASK: usize = TX_BUFFER_SIZE - 1;
const RX_BUFFER_SIZE: usize = 32;
const RX_BUFFER_MASK: usize = RX_BUFFER_SIZE - 1;

#[derive(Copy, Clone)]
pub union Callback {
    pub handler: unsafe extern "C" fn(),
    reserved: usize,
}

#[derive(Copy, Clone)]
pub struct UartInfo {
    pub base_addr: usize,
    pub nvic_irq: u8,
    pub tx_buffer: [u8; TX_BUFFER_SIZE],
    pub tx_enqueued_bytes: EnqCtrType,
    pub tx_dequeued_bytes: EnqCtrType,
    pub rx_callback: Callback,
    pub tx_empty_callback: Callback,
    pub rx_buffer: [u8; RX_BUFFER_SIZE],
    pub rx_enqueued_bytes: EnqCtrType,
    pub rx_dequeued_bytes: EnqCtrType,
}

static mut UARTS: [UartInfo; 6] = [
    UartInfo {
        base_addr: 0x21040000,
        nvic_irq: 4,
        tx_buffer: [0; TX_BUFFER_SIZE],
        tx_enqueued_bytes: 0,
        tx_dequeued_bytes: 0,
        rx_callback: Callback { reserved: 0 },
        tx_empty_callback: Callback { reserved: 0 },
        rx_buffer: [0; RX_BUFFER_SIZE],
        rx_enqueued_bytes: 0,
        rx_dequeued_bytes: 0,
    },
    UartInfo {
        base_addr: 0x38070500,
        nvic_irq: 47,
        tx_buffer: [0; TX_BUFFER_SIZE],
        tx_enqueued_bytes: 0,
        tx_dequeued_bytes: 0,
        rx_callback: Callback { reserved: 0 },
        tx_empty_callback: Callback { reserved: 0 },
        rx_buffer: [0; RX_BUFFER_SIZE],
        rx_enqueued_bytes: 0,
        rx_dequeued_bytes: 0,
    },
    UartInfo {
        base_addr: 0x38080500,
        nvic_irq: 51,
        tx_buffer: [0; TX_BUFFER_SIZE],
        tx_enqueued_bytes: 0,
        tx_dequeued_bytes: 0,
        rx_callback: Callback { reserved: 0 },
        tx_empty_callback: Callback { reserved: 0 },
        rx_buffer: [0; RX_BUFFER_SIZE],
        rx_enqueued_bytes: 0,
        rx_dequeued_bytes: 0,
    },
    UartInfo {
        base_addr: 0x38090500,
        nvic_irq: 55,
        tx_buffer: [0; TX_BUFFER_SIZE],
        tx_enqueued_bytes: 0,
        tx_dequeued_bytes: 0,
        rx_callback: Callback { reserved: 0 },
        tx_empty_callback: Callback { reserved: 0 },
        rx_buffer: [0; RX_BUFFER_SIZE],
        rx_enqueued_bytes: 0,
        rx_dequeued_bytes: 0,
    },
    UartInfo {
        base_addr: 0x380a0500,
        nvic_irq: 59,
        tx_buffer: [0; TX_BUFFER_SIZE],
        tx_enqueued_bytes: 0,
        tx_dequeued_bytes: 0,
        rx_callback: Callback { reserved: 0 },
        tx_empty_callback: Callback { reserved: 0 },
        rx_buffer: [0; RX_BUFFER_SIZE],
        rx_enqueued_bytes: 0,
        rx_dequeued_bytes: 0,
    },
    UartInfo {
        base_addr: 0x380b0500,
        nvic_irq: 63,
        tx_buffer: [0; TX_BUFFER_SIZE],
        tx_enqueued_bytes: 0,
        tx_dequeued_bytes: 0,
        rx_callback: Callback { reserved: 0 },
        tx_empty_callback: Callback { reserved: 0 },
        rx_buffer: [0; RX_BUFFER_SIZE],
        rx_enqueued_bytes: 0,
        rx_dequeued_bytes: 0,
    },
];

pub unsafe fn uart_init(
    id: UartId,
    baudrate: u32,
    databit: u16,
    parity: u16,
    stopbit: u16,
    rx_callback: Callback,
    tx_empty_callback: Callback,
) {
    let unit = match id {
        UartId::UartCM4Debug => &mut UARTS[0],
        UartId::UartIsu0 => &mut UARTS[1],
        UartId::UartIsu1 => &mut UARTS[2],
        UartId::UartIsu2 => &mut UARTS[3],
        UartId::UartIsu3 => &mut UARTS[4],
        UartId::UartIsu4 => &mut UARTS[5],
    };

    // alternative source: https://github.com/MediaTek-Labs/mt3620_m4_software/blob/6f3428cda1fd43190fabcbabf1553f83880aa375/MT3620_M4_Driver/HDL/src/hdl_uart.c#L87
    const UART_CLOCK: u32 = 26000000;
    const UART_LCR_DLAB: u8 = (1 << 7);

    const UART_DLL: usize = 0x00;
    const UART_DLH: usize = 0x04;
    const UART_RATE_STEP: usize = 0x24;
    const UART_STEP_COUNT: usize = 0x28;
    const UART_SAMPLE_COUNT: usize = 0x2c;
    const UART_FRACDIV_M: usize = 0x58;
    const UART_FRACDIV_L: usize = 0x54;
    const UART_LCR: usize = 0x0c;
    const UART_FCR: usize = 0x08;

    let mut uart_lcr: u8 = 0;
    let mut fraction: u8 = 0;
    let mut data: u32 = 0;
    let mut high_speed_div: u32 = 0;
    let mut sample_count: u32 = 0;
    let mut sample_point: u32 = 0;
    let fraction_l_mapping: [u8; 11] = [
        0x00, 0x10, 0x44, 0x92, 0x59, 0xab, 0xb7, 0xdf, 0xff, 0xff, 0xff,
    ];
    let fraction_m_mapping: [u8; 11] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x03,
    ];

    /* Clear fraction */
    write_reg32(unit.base_addr, UART_FRACDIV_L, 0x00);
    write_reg32(unit.base_addr, UART_FRACDIV_M, 0x00);

    /* High speed mode */
    write_reg32(unit.base_addr, UART_RATE_STEP, 0x03);

    /* DLAB start */
    uart_lcr = read_reg32(unit.base_addr, UART_LCR) as u8;
    write_reg32(unit.base_addr, UART_LCR, (uart_lcr | UART_LCR_DLAB) as u32);

    data = UART_CLOCK / baudrate;
    /* divided by 256 */
    high_speed_div = (data >> 8) + 1;

    sample_count = data / high_speed_div - 1;
    /* threshold value */
    if sample_count == 3 {
        sample_point = 0;
    } else {
        sample_point = (sample_count + 1) / 2 - 2;
    }

    /* check uart_clock, prevent calculation overflow */
    fraction =
        (((UART_CLOCK * 10 / baudrate * 10 / high_speed_div - (sample_count + 1) * 100) * 10 + 55)
            / 100) as u8;

    write_reg32(unit.base_addr, UART_DLL, high_speed_div & 0x00ff);
    write_reg32(unit.base_addr, UART_DLH, (high_speed_div >> 8) & 0x00ff);
    write_reg32(unit.base_addr, UART_STEP_COUNT, sample_count);
    write_reg32(unit.base_addr, UART_SAMPLE_COUNT, sample_point);
    write_reg32(
        unit.base_addr,
        UART_FRACDIV_M,
        fraction_m_mapping[fraction as usize] as u32,
    );
    write_reg32(
        unit.base_addr,
        UART_FRACDIV_L,
        fraction_l_mapping[fraction as usize] as u32,
    );

    /* DLAB end */
    write_reg32(unit.base_addr, UART_LCR, uart_lcr as u32);

    const UART_WLS_MASK: u8 = 3 << 0;
    const UART_STOP_MASK: u8 = 1 << 2;
    const UART_PARITY_MASK: u8 = 0x7 << 3;

    let mut control_word: u8 = 0;

    /* DLAB start */
    control_word = read_reg32(unit.base_addr, UART_LCR) as u8;

    control_word &= !UART_WLS_MASK;
    control_word |= databit as u8;
    control_word &= !UART_STOP_MASK;
    control_word |= stopbit as u8;
    control_word &= !UART_PARITY_MASK;
    control_word |= parity as u8;

    /* DLAB End */
    // WriteReg32(unit.baseAddr, UART_LCR, control_word as u32);

    // // Configure UART to use 115200-8-N-1.
    // WriteReg32(unit.baseAddr, 0x0C, 0xBF); // LCR (enable DLL, DLM)
    // WriteReg32(unit.baseAddr, 0x08, 0x10); // EFR (enable enhancement features)
    // WriteReg32(unit.baseAddr, 0x24, 0x3); // HIGHSPEED
    // WriteReg32(unit.baseAddr, 0x04, 0); // Divisor Latch (MS)
    // WriteReg32(unit.baseAddr, 0x00, 1); // Divisor Latch (LS)
    // WriteReg32(unit.baseAddr, 0x28, 224); // SAMPLE_COUNT
    // WriteReg32(unit.baseAddr, 0x2C, 110); // SAMPLE_POINT
    // WriteReg32(unit.baseAddr, 0x58, 0); // FRACDIV_M
    // WriteReg32(unit.baseAddr, 0x54, 223); // FRACDIV_L
    write_reg32(unit.base_addr, 0x0C, 0x03); // LCR (8-bit word length)

    // FCR[RFTL] = 2 -> 12 element RX FIFO trigger
    // FCR[TFTL] = 1 -> 4 element TX FIFO trigger
    // FCR[CLRT] = 1 -> Clear Transmit FIFO
    // FCR[CLRR] = 1 -> Clear Receive FIFO
    // FCR[FIFOE] = 1 -> FIFO Enable
    const FCR: u32 = /*(2u32 << 6) |*/ (1u32 << 4) | (1u32 << 2) | (1u32 << 1) | (1u32 << 0); // 6 = AUTO RTS
    write_reg32(unit.base_addr, 0x08, FCR);

    // If an RX callback was supplied then enable the Receive Buffer Full Interrupt.
    if rx_callback.reserved != 0 {
        unit.rx_callback = rx_callback;
        // IER[ERBGI] = 1 -> Enable Receiver Buffer Full Interrupt + TX_BUFFER_EMPTY
        set_reg32(unit.base_addr, 0x04, 0x01 | 0x02); // UART_INT_RX_BUFFER_FULL | UART_INT_TX_BUFFER_EMPTY
    }

    if tx_empty_callback.reserved != 0 {
        unit.tx_empty_callback = tx_empty_callback
    }

    set_nvic_priority(unit.nvic_irq, UART_PRIORITY);
    enable_nvic_interrupt(unit.nvic_irq);
}

#[no_mangle]
pub unsafe extern "C" fn Uart_HandleIrq4() {
    Uart_HandleIrq(0);
}

#[no_mangle]
pub unsafe extern "C" fn Uart_HandleIrq47() {
    Uart_HandleIrq(1);
}

#[no_mangle]
pub unsafe extern "C" fn Uart_HandleIrq51() {
    Uart_HandleIrq(2);
}

#[no_mangle]
pub unsafe extern "C" fn Uart_HandleIrq55() {
    Uart_HandleIrq(3);
}

#[no_mangle]
pub unsafe extern "C" fn Uart_HandleIrq59() {
    Uart_HandleIrq(4);
}

#[no_mangle]
pub unsafe extern "C" fn Uart_HandleIrq63() {
    Uart_HandleIrq(5);
}

#[no_mangle]
pub unsafe extern "C" fn Uart_HandleIrq67() {
    Uart_HandleIrq(5);
}

#[no_mangle]
unsafe extern "C" fn Uart_HandleIrq(id: usize) {
    let unit = &mut UARTS[id];

    let mut iir_id: u32 = 0;
    'outer: loop {
        // Interrupt Identification Register[IIR_ID]
        iir_id = read_reg32(unit.base_addr, 0x08) & 0x1F;
        match iir_id {
        0x01 =>  // No interrupt pending
            break
            // The TX FIFO can accept more data.
            ,
        0x02 => { // TX Holding Register Empty Interrupt
            let mut local_enqueued_bytes = unit.tx_enqueued_bytes;
            let mut local_dequeued_bytes = unit.tx_dequeued_bytes;

            // TX_OFFSET, holds number of bytes in TX FIFO.
            let tx_offset: u32 = read_reg32(unit.base_addr, 0x6C);
            let mut space_in_tx_fifo = TX_FIFO_DEPTH - tx_offset;

            while local_dequeued_bytes != local_enqueued_bytes && space_in_tx_fifo > 0 {
                let tx_idx = local_dequeued_bytes & TX_BUFFER_MASK as u16;
                // TX Holding Register
                write_reg32(unit.base_addr, 0x00, unit.tx_buffer[tx_idx as usize] as u32);

                local_dequeued_bytes = local_dequeued_bytes + 1;
                space_in_tx_fifo = space_in_tx_fifo - 1;
            }

            unit.tx_dequeued_bytes = local_dequeued_bytes;

            // If sent all enqueued data then disable TX interrupt.
            if local_enqueued_bytes == local_dequeued_bytes {
                // Interrupt Enable Register
                clear_reg32(unit.base_addr, 0x04, 0x02);

                // wait for all data written (i.e. tx buffer empty) - enabling the irq doesn't work for some reason?
                loop {
                    let line_status = read_reg32(unit.base_addr, 0x14);

                    if line_status & 0b1000000 != 0 {
                        if unit.tx_empty_callback.reserved != 0 {
                            (unit.tx_empty_callback.handler)();
                        }

                        break 'outer;
                    }
                }
            }
            break;
        },

        // Read from the FIFO if it has passed its trigger level, or if a timeout
        // has occurred, meaning there is unread data still in the FIFO.
        0x0C |  // RX Data Timeout Interrupt
        0x04 => { // RX Data Received Interrupt
            let mut local_enqueued_bytes = unit.rx_enqueued_bytes;
            let local_dequeued_bytes = unit.rx_dequeued_bytes;

            let mut avail_space: EnqCtrType = 0;
            if local_enqueued_bytes >= local_dequeued_bytes {
                avail_space = (RX_BUFFER_SIZE - (local_enqueued_bytes as usize - local_dequeued_bytes as usize)) as u16;
            }
            // If counter wrapped around, work out true remaining space.
            else {
                avail_space = (local_dequeued_bytes & RX_BUFFER_MASK as u16) - local_enqueued_bytes;
            }

            // LSR[0] = 1 -> Data Ready
            while avail_space > 0 && (read_reg32(unit.base_addr, 0x14) & 0x01)!=0 {
                let idx: EnqCtrType = local_enqueued_bytes & RX_BUFFER_MASK as u16;
                // RX Buffer Register
                unit.rx_buffer[idx as usize] = read_reg32(unit.base_addr, 0x00) as u8;

                local_enqueued_bytes = local_enqueued_bytes + 1;
                avail_space = avail_space - 1;
            }

            unit.rx_enqueued_bytes = local_enqueued_bytes;

            if unit.rx_callback.reserved != 0 {
                (unit.rx_callback.handler)();
            }

            break;
        },
        _ => {
        }

        }

        if iir_id == 0x01 {
            break;
        }
    }
}

pub fn uart_enqueue_data(id: UartId, data: *const u8, length: usize) -> bool {
    let unit = unsafe {
        match id {
            UartId::UartCM4Debug => &mut UARTS[0],
            UartId::UartIsu0 => &mut UARTS[1],
            UartId::UartIsu1 => &mut UARTS[2],
            UartId::UartIsu2 => &mut UARTS[3],
            UartId::UartIsu3 => &mut UARTS[4],
            UartId::UartIsu4 => &mut UARTS[5],
        }
    };

    let mut local_enqueued_bytes = unit.tx_enqueued_bytes;
    let local_dequeued_bytes = unit.tx_dequeued_bytes;

    let mut avail_space: EnqCtrType = 0;
    if local_enqueued_bytes >= local_dequeued_bytes {
        avail_space = (TX_BUFFER_SIZE
            - (local_enqueued_bytes as usize - local_dequeued_bytes as usize))
            as u16;
    }
    // If counter wrapped around, work out true remaining space.
    else {
        avail_space = (local_dequeued_bytes & TX_BUFFER_MASK as u16) - local_enqueued_bytes;
    }

    // If no available space then do not enable TX interrupt.
    if avail_space == 0 {
        return false;
    }

    // Copy as much data as possible from the message to the buffer.
    // Any unqueued data will be lost.
    let write_all = avail_space >= length as u16;
    let mut bytes_to_write: EnqCtrType = if write_all {
        length as u16
    } else {
        avail_space
    };

    let mut i: isize = 0;
    loop {
        bytes_to_write -= 1;

        let idx = local_enqueued_bytes & TX_BUFFER_MASK as u16;

        unsafe {
            unit.tx_buffer[idx as usize] = *(data.offset(i));
            local_enqueued_bytes += 1;
        }

        i += 1;
        if bytes_to_write == 0 {
            break;
        }
    }

    // Block IRQs here because the the UART IRQ could already be enabled, and run
    // between updating txEnqueuedBytes and re-enabling the IRQ here. If that happened,
    // the IRQ could exhaust the software buffer and disable the TX interrupt, only
    // for it to be re-enabled here, in which case it would not get cleared because
    // there was no data to write to the TX FIFO.
    let prev_pri_base = block_irqs();
    unit.tx_enqueued_bytes = local_enqueued_bytes;
    // IER[ETBEI] = 1 -> Enable Transmitter Buffer Empty Interrupt
    set_reg32(unit.base_addr, 0x04, 0x02);
    restore_irqs(prev_pri_base);

    return write_all;
}

pub fn uart_dequeue_data(id: UartId, data: *mut u8, length: usize) -> usize {
    let unit = unsafe {
        match id {
            UartId::UartCM4Debug => &mut UARTS[0],
            UartId::UartIsu0 => &mut UARTS[1],
            UartId::UartIsu1 => &mut UARTS[2],
            UartId::UartIsu2 => &mut UARTS[3],
            UartId::UartIsu3 => &mut UARTS[4],
            UartId::UartIsu4 => &mut UARTS[5],
        }
    };

    let local_enqueued_bytes: EnqCtrType = unit.rx_enqueued_bytes;
    let local_dequeued_bytes: EnqCtrType = unit.rx_dequeued_bytes;

    let mut avail_data: EnqCtrType = 0;
    if local_enqueued_bytes >= local_dequeued_bytes {
        avail_data = local_enqueued_bytes - local_dequeued_bytes;
    } else {
        // Wraparound occurred so work out the true available data.
        avail_data = (RX_BUFFER_SIZE
            - ((local_dequeued_bytes as usize & RX_BUFFER_MASK) - local_enqueued_bytes as usize))
            as u16;
    }

    // This check is required to distinguish an empty buffer from a full buffer, because
    // in both cases the enqueue and dequeue indices point to the same index.
    if avail_data == 0 {
        return 0;
    }

    let enqueue_index: EnqCtrType = local_enqueued_bytes & RX_BUFFER_MASK as u16;
    let dequeue_index: EnqCtrType = local_dequeued_bytes & RX_BUFFER_MASK as u16;

    unsafe {
        // If the available data does not wraparound use one memcpy...
        if enqueue_index > dequeue_index {
            core::ptr::copy(
                unit.rx_buffer.as_ptr().offset(dequeue_index as isize),
                data,
                avail_data as usize,
            );
        //__builtin_memcpy(buffer, &unit->rxBuffer[dequeueIndex], availData);
        } else {
            // ...otherwise copy data from end of buffer, then from start.
            let bytes_from_end = RX_BUFFER_SIZE - dequeue_index as usize;

            core::ptr::copy(
                unit.rx_buffer.as_ptr().offset(dequeue_index as isize),
                data,
                bytes_from_end as usize,
            );
            //__builtin_memcpy(buffer, &unit->rxBuffer[dequeueIndex], bytesFromEnd);
            core::ptr::copy(
                unit.rx_buffer.as_ptr(),
                data.offset(bytes_from_end as isize),
                enqueue_index as usize,
            );
            //__builtin_memcpy(buffer + bytesFromEnd, &unit->rxBuffer[0], enqueueIndex);
        }
    }

    unit.rx_dequeued_bytes += avail_data;
    return avail_data as usize;
}

pub fn set_rts(id: UartId, on: bool) {
    let unit = unsafe {
        match id {
            UartId::UartCM4Debug => &mut UARTS[0],
            UartId::UartIsu0 => &mut UARTS[1],
            UartId::UartIsu1 => &mut UARTS[2],
            UartId::UartIsu2 => &mut UARTS[3],
            UartId::UartIsu3 => &mut UARTS[4],
            UartId::UartIsu4 => &mut UARTS[5],
        }
    };

    const MCR: usize = 0x10;

    if on {
        set_reg32(unit.base_addr, MCR, 0b10);
    } else {
        clear_reg32(unit.base_addr, MCR, 0b10);
    }
}
