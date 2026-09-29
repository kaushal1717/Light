//! A compressed-expansion PDF: one page whose content stream is a zlib stream inflating to just
//! under 1 GiB of spaces from ~6.8 MB on disk -- under the 16 MiB file limit, so only a bound on
//! the parser's own memory can stop it. Built bit by bit (fixed-Huffman deflate: one literal,
//! then `length 258, distance 1` repeated), not by compressing a gigabyte: milliseconds to make.

const GIB: u64 = 1 << 30;

#[derive(Default)]
struct Bits {
    out: Vec<u8>,
    acc: u64,
    len: u32,
}

impl Bits {
    /// Deflate packs header fields LSB-first and Huffman codes MSB-first.
    fn put(&mut self, value: u32, len: u32, huffman: bool) {
        let value = match huffman {
            true => (0..len).fold(0, |r, i| (r << 1) | ((value >> i) & 1)),
            false => value,
        };
        self.acc |= u64::from(value) << self.len;
        self.len += len;
        while self.len >= 8 {
            self.out.push(self.acc as u8);
            (self.acc, self.len) = (self.acc >> 8, self.len - 8);
        }
    }
}

fn zlib_of_spaces() -> Vec<u8> {
    let runs = (GIB - 1) / 258;
    let mut bits = Bits::default();
    bits.put(1, 1, false); // BFINAL
    bits.put(1, 2, false); // BTYPE = fixed Huffman
    bits.put(0x30 + u32::from(b' '), 8, true); // literal ' '
    for _ in 0..runs {
        bits.put(0xC5, 8, true); // symbol 285: length 258
        bits.put(0, 5, true); // distance code 0: distance 1
    }
    bits.put(0, 7, true); // symbol 256: end of block, then zero bits to pad the last byte
    bits.put(0, 7, false);
    // Adler-32 of n bytes all equal to c, in closed form: a = 1 + nc, b = n + c*n(n+1)/2.
    let (n, c, m) = (u128::from(1 + 258 * runs), u128::from(b' '), 65_521u128);
    let (a, b) = ((1 + n * c) % m, (n + c * n * (n + 1) / 2) % m);
    let mut zlib = vec![0x78, 0x01];
    zlib.extend_from_slice(&bits.out);
    zlib.extend_from_slice(&(((b << 16) | a) as u32).to_be_bytes());
    zlib
}

pub fn expansion_pdf() -> Vec<u8> {
    let stream = zlib_of_spaces();
    let mut pdf = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::new();
    let head = format!(
        "<< /Length {} /Filter /FlateDecode >>\nstream\n",
        stream.len()
    );
    let objects: [&[&[u8]]; 4] = [
        &[b"<< /Type /Catalog /Pages 2 0 R >>"],
        &[b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>"],
        &[b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R >>"],
        &[head.as_bytes(), &stream, b"\nendstream"],
    ];
    for (i, parts) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        pdf.extend_from_slice(format!("{} 0 obj\n", i + 1).as_bytes());
        parts.iter().for_each(|part| pdf.extend_from_slice(part));
        pdf.extend_from_slice(b"\nendobj\n");
    }
    let xref_at = pdf.len();
    pdf.extend_from_slice(b"xref\n0 5\n0000000000 65535 f \n");
    for offset in offsets {
        pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    let tail = format!("trailer\n<< /Size 5 /Root 1 0 R >>\nstartxref\n{xref_at}\n%%EOF\n");
    pdf.extend_from_slice(tail.as_bytes());
    pdf
}
