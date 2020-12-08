use super::gpio::*;

/// <summary>
/// There are two buffers, inbound and outbound, which are used to track
/// how much data has been written to, and read from, each shared buffer.
/// </summary>
#[repr(C)]
pub struct BufferHeader {
    /// <summary>
    /// <para>Enqueue function uses this value to store the last position written to
    /// by the real-time capable application.</para>
    /// <para>Dequeue function uses this value to find the last position written to by
    /// the high-level application.</summary>
    write_position: u32,
    /// <summary>
    /// <para>Enqueue function uses this value to find the last position read from by the
    /// high-level applicaton.</para>
    /// <para>Dequeue function uses this value to store the last position read from by
    /// the real-time application.</para>
    read_position: u32,
    /// <summary>Reserved for alignment.</summary>
    reserved: [u32; 14],
}

/// <summary>Blocks inside the shared buffer have this alignment.</summary>
const RINGBUFFER_ALIGNMENT: usize = 16;

/// <summary>
/// <para>Gets the inbound and outbound buffers used to communicate with the high-level
/// application.  This function blocks until that data is available from the mailbox.</para>
/// <para>The retrieved pointers are then supplied to <see cref="EnqueueData" /> and
/// <see cref="DequeueData" />.</para>
/// </summary>
/// <param name="outbound">On success, this points to the buffer which the real-time capable
/// application uses to send messages to the high-level application.</param>
/// <param name="inbound">On success, this points to the buffer which the real-time capable
/// application uses to receive messages from the high-level application.</param>
/// <param name="bufSize">On success, this contains the buffer size in bytes.</param>
/// <returns>0 on success, -1 on failure.</returns>
pub fn get_intercore_buffers() -> Result<(*mut BufferHeader, *mut BufferHeader, u32), &'static str>
{
    // Wait for the mailbox to be set up.
    let mut base_read: usize = 0;
    let mut base_write: usize = 0;
    loop {
        let mut cmd: u32 = 0;
        let mut data: u32 = 0;

        receive_message(&mut cmd, &mut data);
        if cmd == 0xba5e0001 {
            base_write = data as usize;
        } else if cmd == 0xba5e0002 {
            base_read = data as usize;
        } else if cmd == 0xba5e0003 {
            break;
        }
    }

    let inbound_buffer_size = get_buffer_size(base_read);
    let outbound_buffer_size = get_buffer_size(base_write);

    if inbound_buffer_size != outbound_buffer_size {
        return Err("GetIntercoreBuffers: Mismatched buffer sizes");
    }

    if inbound_buffer_size <= core::mem::size_of::<BufferHeader>() as u32 {
        return Err("GetIntercoreBuffers: buffer size smaller than header");
    }

    let outbound: *mut BufferHeader;
    let inbound: *mut BufferHeader;
    let buf_size: u32;

    buf_size = inbound_buffer_size - core::mem::size_of::<BufferHeader>() as u32;
    inbound = get_buffer_header(base_read) as *mut BufferHeader;
    outbound = get_buffer_header(base_write) as *mut BufferHeader;

    return Ok((inbound, outbound, buf_size));
}

/// <summary>
/// Add data to the shared buffer, to be read by the high-level application.
/// </summary>
/// <param name="outbound">The outbound buffer, as obtained from <see cref="GetIntercoreBuffers" />.
/// </param>
/// <param name="inbound">The inbound buffer, as obtained from <see cref="GetIntercoreBuffers" />.
/// </param>
/// <param name="bufSize">
/// The total buffer size, as obtained from <see cref="GetIntercoreBuffers" />.
/// </param>
/// <param name="src">Start of data to write to buffer.</param>
/// <param name="dataSize">Length of data to write to buffer in bytes.</param>
/// <returns>0 if able to enqueue the data, -1 otherwise.</returns>
pub fn enqueue_data(
    inbound: *const BufferHeader,
    outbound: *mut BufferHeader,
    buf_size: usize,
    src: *const u8,
    data_size: usize,
) -> Result<&'static str, &'static str> {
    unsafe {
        let remote_read_position = (*inbound).read_position;
        let mut local_write_position = (*outbound).write_position;

        if remote_read_position >= buf_size as u32 {
            return Err("EnqueueData: remoteReadPosition invalid");
        }

        // If the read pointer is behind the write pointer, then the free space wraps around.
        let mut avail_space = 0i32;
        if remote_read_position <= local_write_position {
            avail_space =
                remote_read_position as i32 - local_write_position as i32 + buf_size as i32;
        } else {
            avail_space = remote_read_position as i32 - local_write_position as i32;
        }

        // If there isn't enough space to enqueue a block, then abort the operation.
        if avail_space < (core::mem::size_of::<u32>() + data_size + RINGBUFFER_ALIGNMENT) as i32 {
            return Err("EnqueueData: not enough space to enqueue block");
        }

        // Write up to end of buffer. If the block ends before then, only write up to the end of the
        // block.
        let data_to_end = buf_size as isize - local_write_position as isize;

        // There must be enough space between the write pointer and the end of the buffer to store the
        // block size as a contiguous 4-byte value. The remainder of message can wrap around.
        if data_to_end < core::mem::size_of::<u32>() as isize {
            return Err("EnqueueData: not enough space for block size");
        }

        let mut write_to_end = core::mem::size_of::<u32>() + data_size;
        if data_to_end < write_to_end as isize {
            write_to_end = data_to_end as usize;
        }

        // Write block size to first word in block.
        let tmp = data_area_offset32(outbound, local_write_position as isize);
        *tmp = data_size as u32;
        write_to_end -= core::mem::size_of::<u32>();

        let src8 = src;
        let dest8 = data_area_offset8(
            outbound,
            local_write_position as isize + core::mem::size_of::<u32>() as isize,
        );

        core::ptr::copy(src8, dest8, write_to_end as usize);
        //__builtin_memcpy(dest8, src8, writeToEnd);

        core::ptr::copy(
            src8.offset(write_to_end as isize),
            data_area_offset8(outbound, 0),
            (data_size - write_to_end) as usize,
        );
        //__builtin_memcpy(DataAreaOffset8(outbound, 0), src8 + writeToEnd, dataSize - writeToEnd);

        // Advance write position.
        local_write_position = round_up(
            local_write_position + core::mem::size_of::<u32>() as u32 + data_size as u32,
            RINGBUFFER_ALIGNMENT as u32,
        );
        if local_write_position >= buf_size as u32 {
            local_write_position -= buf_size as u32;
        }

        (*outbound).write_position = local_write_position;

        // SW_TX_INT_PORT[0] = 1 -> indicate message received.
        write_reg32(MAILBOX_BASE, 0x14, 1u32 << 0);
        return Ok("Ok");
    }
}

/// <summary>
/// Remove data from the shared buffer, which has been written by the high-level application.
/// </summary>
/// <param name="outbound">The outbound buffer, as obtained from <see cref="GetIntercoreBuffers" />.
/// </param>
/// <param name="inbound">The inbound buffer, as obtained from <see cref="GetIntercoreBuffers" />.
/// </param>
/// <param name="bufSize">Total size of shared buffer in bytes.</param>
/// <param name="dest">Data from the shared buffer is copied into this buffer.</param>
/// <param name="dataSize">On entry, contains maximum size of destination buffer in bytes.
/// On exit, contains the actual number of bytes which were written to the destination buffer.
/// </param>
/// <returns>0 if able to dequeue the data, -1 otherwise.</returns>
// int DequeueData(BufferHeader *outbound, BufferHeader *inbound, uint32_t bufSize, void *dest, uint32_t *dataSize);
pub fn dequeue_data(
    outbound: *mut BufferHeader,
    inbound: *const BufferHeader,
    buf_size: usize,
    dest: *mut u8,
    max_data_size: usize,
) -> Result<usize, (&'static str, usize)> {
    unsafe {
        let mut remote_write_position = (*inbound).write_position;
        let mut local_read_position = (*outbound).read_position;

        if remote_write_position > buf_size as u32 {
            return Err(("DequeueData: remoteWritePosition invalid", 0));
        }

        let mut avail_data: isize = 0;
        // If data is contiguous in buffer then difference between write and read positions...
        if remote_write_position >= local_read_position {
            avail_data = remote_write_position as isize - local_read_position as isize;
        } else {
            // ...else data wraps around end and resumes at start of buffer
            avail_data =
                remote_write_position as isize - local_read_position as isize + buf_size as isize;
        }

        // There must be at least four contiguous bytes to hold the block size.
        if avail_data < core::mem::size_of::<u32>() as isize {
            return Err(("DequeueData: availData < 4 bytes", 0));
        }

        let data_to_end = buf_size as isize - local_read_position as isize;
        if data_to_end < core::mem::size_of::<u32>() as isize {
            return Err(("DequeueData: dataToEnd < 4 bytes", 0));
        }

        let block_size = *data_area_offset32(inbound, local_read_position as isize);

        // Ensure the block size is no greater than the available data.
        if block_size as isize + core::mem::size_of::<u32>() as isize > avail_data {
            return Err(("DequeueData: message size greater than available data", 0));
        }

        // Abort if the caller-supplied buffer is not large enough to hold the message.
        if block_size > max_data_size as u32 {
            return Err((
                "DequeueData: message too large for buffer",
                block_size as usize,
            ));
        }

        // Tell the caller the actual block size.
        //*dataSize = blockSize;

        // Read up to the end of the buffer. If the block ends before then, only read up to the end
        // of the block.
        let mut read_from_end = data_to_end - core::mem::size_of::<u32>() as isize;
        if block_size < read_from_end as u32 {
            read_from_end = block_size as isize;
        }

        let src8 = data_area_offset8(
            inbound,
            local_read_position as isize + core::mem::size_of::<u32>() as isize,
        );
        let dest8 = dest;

        core::ptr::copy(src8, dest8, read_from_end as usize);
        //__builtin_memcpy(dest8, src8, readFromEnd);

        // If block wrapped around the end of the buffer, then read remainder from start.

        core::ptr::copy(
            data_area_offset8(inbound, 0),
            dest8.offset(read_from_end as isize),
            block_size as usize - read_from_end as usize,
        );
        //__builtin_memcpy(dest8 + readFromEnd, DataAreaOffset8(inbound, 0), blockSize - readFromEnd);

        // Round read position to next aligned block, and wraparound end of buffer if required.
        local_read_position = round_up(
            local_read_position + core::mem::size_of::<u32>() as u32 + block_size,
            RINGBUFFER_ALIGNMENT as u32,
        );
        if local_read_position > buf_size as u32 {
            local_read_position -= buf_size as u32;
        }

        (*outbound).read_position = local_read_position;

        // SW_TX_INT_PORT[1] = 1 -> indicate message received.
        write_reg32(MAILBOX_BASE, 0x14, 1u32 << 1);

        return Ok(block_size as usize);
    }
}

const MAILBOX_BASE: usize = 0x21050000;

fn receive_message(command: *mut u32, data: *mut u32) {
    // FIFO_POP_CNT
    while read_reg32(MAILBOX_BASE, 0x58) == 0 {
        // empty.
    }

    unsafe {
        // DATA_POP0
        *data = read_reg32(MAILBOX_BASE, 0x54);
        // CMD_POP0
        *command = read_reg32(MAILBOX_BASE, 0x50);
    }
}

fn get_buffer_size(buffer_base: usize) -> u32 {
    return 1u32 << (buffer_base & 0x1F);
}

fn get_buffer_header(buffer_base: usize) -> *mut BufferHeader {
    return (buffer_base & !0x1F) as *mut BufferHeader;
}

fn data_area_offset8(header: *const BufferHeader, offset: isize) -> *mut u8 {
    unsafe {
        // Data storage area following header in buffer.
        let data_start = header.offset(1) as *mut u8;

        // Offset within data storage area.
        data_start.offset(offset) as *mut u8
    }
}

fn data_area_offset32(header: *const BufferHeader, offset: isize) -> *mut u32 {
    data_area_offset8(header, offset) as *mut u32
}

fn round_up(value: u32, alignment: u32) -> u32 {
    // alignment must be a power of two.
    (value + (alignment - 1)) & !(alignment - 1)
}
