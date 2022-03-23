use core::arch::asm;

const COLORS: [u32; 80] = [
    0xF9FAFB, 0xF3F4F6, 0xE5E7EB, 0xD1D5DB, 0x9CA3AF, 0x6B7280, 0x4B5563, 0x374151, 0x1F2937,
    0x111827, 0xFEF2F2, 0xFEE2E2, 0xFECACA, 0xFCA5A5, 0xF87171, 0xEF4444, 0xDC2626, 0xB91C1C,
    0x991B1B, 0x7F1D1D, 0xFFFBEB, 0xFEF3C7, 0xFDE68A, 0xFCD34D, 0xFBBF24, 0xF59E0B, 0xD97706,
    0xB45309, 0x92400E, 0x78350F, 0xECFDF5, 0xD1FAE5, 0xA7F3D0, 0x6EE7B7, 0x34D399, 0x10B981,
    0x059669, 0x047857, 0x065F46, 0x064E3B, 0xEFF6FF, 0xDBEAFE, 0xBFDBFE, 0x93C5FD, 0x60A5FA,
    0x3B82F6, 0x2563EB, 0x1D4ED8, 0x1E40AF, 0x1E3A8A, 0xEEF2FF, 0xE0E7FF, 0xC7D2FE, 0xA5B4FC,
    0x818CF8, 0x6366F1, 0x4F46E5, 0x4338CA, 0x3730A3, 0x312E81, 0xF5F3FF, 0xEDE9FE, 0xDDD6FE,
    0xC4B5FD, 0xA78BFA, 0x8B5CF6, 0x7C3AED, 0x6D28D9, 0x5B21B6, 0x4C1D95, 0xFDF2F8, 0xFCE7F3,
    0xFBCFE8, 0xF9A8D4, 0xF472B6, 0xEC4899, 0xDB2777, 0xBE185D, 0x9D174D, 0x831843,
];

pub fn draw(fb_addr: *mut u32, resolution: (usize, usize), box_size: isize) {
    let width = resolution.0 as isize;
    let height = resolution.1 as isize;
    let x_range = (box_size, width - 1 - box_size);
    let y_range = (box_size, height - 1 - box_size);

    let in_range = |x: isize, y: isize| -> bool {
        return x > x_range.0 && x < x_range.1
            && y > y_range.0 && y < y_range.1;
    };

    let color_len = COLORS.len();

    let mut row: isize = x_range.0;
    let mut col: isize = y_range.0;

    let mut row_incr: isize = 1;
    let mut col_incr: isize = 2;
    let mut color = 0;

    loop {
        row += row_incr;
        col += col_incr;

        if in_range(col, row) {
            for i in -box_size..box_size + 1 {
                for j in -box_size..box_size + 1 {
                    unsafe {
                        *(fb_addr as *mut u32).offset(((row + i) * width + col + j) as isize) =
                            COLORS[color];
                        *(fb_addr as *mut u32)
                            .offset(((height - row - i) * width + (width - col - j)) as isize) =
                            COLORS[color];
                    }
                }
            }
        }

        if col <= x_range.0 || col >= x_range.1 {
            col_incr = -col_incr;
        }

        if row <= y_range.0 || row >= y_range.1 {
            row_incr = -row_incr;
        }

        color = (color + 1) % color_len;

        for _ in 0..500_0000 {
            unsafe {
                asm!("nop");
            }
        }
    }
}
