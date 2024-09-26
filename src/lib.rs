enum ColorSpace {
    SRGB,
    Linear,
}

enum Channels {
    RGB,
    RGBA,
}

#[derive(Clone)]
enum Chunk {
    RGB(PixelRGB),
    RGBA(PixelRGBA),
    Index(u8),
    Diff(DiffRGB),
    Luma(Luma),
    Run(u8),
}

#[derive(Clone)]
struct Luma(u8, u8, u8);

#[derive(Clone)]
struct DiffRGB(u8, u8, u8);

pub struct QOIImage {
    width: u32,
    height: u32,
    channels: Channels,
    color_space: ColorSpace,
    data: Vec<Chunk>,
}

impl QOIImage {
    pub fn from_qoi_file<R: std::io::Read>(
        mut source: std::io::Bytes<R>,
    ) -> Result<QOIImage, &'static str> {
        let mut header = [0u8; 14];
        for i in 0..14 {
            if let Some(Ok(x)) = source.next() {
                header[i] = x;
            } else {
                return Err("Malformed input: incomplete header");
            }
        }

        if Ok("qoif".to_string()) != String::from_utf8(header[0..4].to_vec()) {
            return Err("Malformed input: magic bytes not found");
        }

        let width = u32::from_be_bytes(header[4..8].try_into().unwrap());
        let height = u32::from_be_bytes(header[8..12].try_into().unwrap());
        let channels = match header[12] {
            3u8 => Channels::RGB,
            4u8 => Channels::RGBA,
            _ => {
                return Err("Malformed input: invalid channels data");
            }
        };
        let color_space = match header[13] {
            0u8 => ColorSpace::SRGB,
            1u8 => ColorSpace::Linear,
            _ => {
                return Err("Malformed input: invalid channels data");
            }
        };

        let mut data: Vec<Chunk> = Vec::new();

        // build the chunks
        let mut zeroes_so_far = 0;
        while let Some(Ok(current_chunk)) = source.next() {
            match current_chunk {
                0b11111111 => {
                    let r = {
                        let r = source.next();
                        if r.is_none() {
                            return Err("Malformed input: reached end of file abruptly");
                        } else {
                            let r = r.unwrap();
                            if r.is_err() {
                                return Err("Malformed input: reached end of file abruptly");
                            } else {
                                r.unwrap()
                            }
                        }
                    };

                    let g = {
                        let g = source.next();
                        if g.is_none() {
                            return Err("Malformed input: reached end of file abruptly");
                        } else {
                            let g = g.unwrap();
                            if g.is_err() {
                                return Err("Malformed input: reached end of file abruptly");
                            } else {
                                g.unwrap()
                            }
                        }
                    };

                    let b = {
                        let b = source.next();
                        if b.is_none() {
                            return Err("Malformed input: reached end of file abruptly");
                        } else {
                            let b = b.unwrap();
                            if b.is_err() {
                                return Err("Malformed input: reached end of file abruptly");
                            } else {
                                b.unwrap()
                            }
                        }
                    };

                    let a = {
                        let a = source.next();
                        if a.is_none() {
                            return Err("Malformed input: reached end of file abruptly");
                        } else {
                            let a = a.unwrap();
                            if a.is_err() {
                                return Err("Malformed input: reached end of file abruptly");
                            } else {
                                a.unwrap()
                            }
                        }
                    };

                    data.push(Chunk::RGBA(PixelRGBA(r, g, b, a)));
                    zeroes_so_far = 0;
                }
                0b11111110 => {
                    let r = {
                        let r = source.next();
                        if r.is_none() {
                            return Err("Malformed input: reached end of file abruptly");
                        } else {
                            let r = r.unwrap();
                            if r.is_err() {
                                return Err("Malformed input: reached end of file abruptly");
                            } else {
                                r.unwrap()
                            }
                        }
                    };

                    let g = {
                        let g = source.next();
                        if g.is_none() {
                            return Err("Malformed input: reached end of file abruptly");
                        } else {
                            let g = g.unwrap();
                            if g.is_err() {
                                return Err("Malformed input: reached end of file abruptly");
                            } else {
                                g.unwrap()
                            }
                        }
                    };

                    let b = {
                        let b = source.next();
                        if b.is_none() {
                            return Err("Malformed input: reached end of file abruptly");
                        } else {
                            let b = b.unwrap();
                            if b.is_err() {
                                return Err("Malformed input: reached end of file abruptly");
                            } else {
                                b.unwrap()
                            }
                        }
                    };

                    data.push(Chunk::RGB(PixelRGB(r, g, b)));
                    zeroes_so_far = 0;
                }
                n if n >> 6 == 0b11 => {
                    data.push(Chunk::Run(n & 0b00111111));
                    zeroes_so_far = 0;
                }
                n if n >> 6 == 0b00 => {
                    if n == 0 {
                        zeroes_so_far += 1;
                    }
                    if n == 1 && zeroes_so_far == 7 {
                        for _ in 0..7 {
                            data.pop();
                        }
                        break;
                    }
                    data.push(Chunk::Index(n & 0b00111111));
                }
                n if n >> 6 == 0b01 => {
                    let r: u8 = (n & 0b00110000) >> 4;
                    let g: u8 = (n & 0b00001100) >> 2;
                    let b: u8 = n & 0b11;
                    data.push(Chunk::Diff(DiffRGB(r, g, b)));
                    zeroes_so_far = 0;
                }
                n if n >> 6 == 0b10 => {
                    let dg = n & 0b00111111;
                    if let Some(Ok(next_byte)) = source.next() {
                        // dg = 6 bit green channel difference from the previous pixel -32..31
                        // next_byte = ((current_pixel.r - prev_pixel.r) - (current_pixel.g - prev_pixel.g) << 4)
                        //           + ((current_pixel.b - prev_pixel.b) - (current_pixel.g - prev_pixel.g)
                        let dr_dg = next_byte >> 4;
                        let db_dg = next_byte & 0b1111;
                        data.push(Chunk::Luma(Luma(dg, dr_dg, db_dg)));
                        zeroes_so_far = 0;
                    } else {
                        return Err("Malformed input: reached end of file abruptly");
                    }
                }
                _ => return Err("Malformed input: invalid channels data"),
            }
        }

        Ok(QOIImage {
            width,
            height,
            channels,
            color_space,
            data,
        })
    }

    pub fn serialize(&self) -> Vec<u8> {
        // build header
        let mut header: Vec<u8> = Vec::with_capacity(14);
        header.extend_from_slice(&vec![0x71, 0x6F, 0x69, 0x66]);
        header.extend_from_slice(&self.width.to_be_bytes());
        header.extend_from_slice(&self.height.to_be_bytes());
        let channels: u8 = match self.channels {
            Channels::RGB => 3,
            Channels::RGBA => 4,
        };
        header.push(channels);
        let color_space: u8 = match self.color_space {
            ColorSpace::Linear => 1,
            ColorSpace::SRGB => 0,
        };
        header.push(color_space);

        // build image data
        let image = self
            .data
            .iter()
            .fold(Vec::new(), |mut data, chunk| -> Vec<u8> {
                match chunk {
                    &Chunk::RGB(PixelRGB(r, g, b)) => {
                        data.extend_from_slice(&[0b11111110, r, g, b]);
                        data
                    }
                    &Chunk::RGBA(PixelRGBA(r, g, b, a)) => {
                        data.extend_from_slice(&[0b11111111, r, g, b, a]);
                        data
                    }
                    &Chunk::Index(i) => {
                        data.push(i);
                        data
                    }
                    &Chunk::Diff(DiffRGB(r, g, b)) => {
                        data.push((0b01 << 6) + (r << 4) + (g << 2) + b);
                        data
                    }
                    &Chunk::Luma(Luma(dg, dr_dg, db_dg)) => {
                        data.push((0b10 << 6) + dg);
                        data.push((dr_dg << 4) + db_dg);
                        data
                    }
                    &Chunk::Run(n) => {
                        data.push((0b11 << 6) + n);
                        data
                    }
                }
            });

        let mut res = header;
        res.extend_from_slice(&image);
        res.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 1]);
        res
    }

    pub fn to_rgba_mat(&self) -> Vec<Vec<PixelRGBA>> {
        // images are encoded row by row, left to right, top to bottom
        // an image is complete when all pixels specified by width*height have been covered.
        let width = self.width as usize;
        let height = self.height as usize;

        // the decoder starts with PixelRGBA(0,0,0,255) as the previous pixel value.
        let mut prev_px = PixelRGBA(0, 0, 0, 255);

        // a running array[64] of PixelRGBA(0,0,0,0) is maintained by the decoder.
        // every pixel value seen by the decoder is put into the array at i=(r*3 + g*5 + b*7 + a*11) % 64
        let mut hash = [PixelRGBA(0, 0, 0, 0); 64];

        let mut img = Vec::with_capacity(width * height);
        for chunk in &self.data {
            match chunk {
                &Chunk::RGB(PixelRGB(r, g, b)) => {
                    // simple cast to PixelRGBA
                    img.push(PixelRGBA(r, g, b, prev_px.3));
                    hash[(r as usize * 3
                        + g as usize * 5
                        + b as usize * 7
                        + prev_px.3 as usize * 11)
                        % 64] = PixelRGBA(r, g, b, prev_px.3);
                    prev_px = PixelRGBA(r, g, b, prev_px.3);
                }
                &Chunk::RGBA(px) => {
                    img.push(px);
                    hash[(px.0 as usize * 3
                        + px.1 as usize * 5
                        + px.2 as usize * 7
                        + px.3 as usize * 11)
                        % 64] = px;
                    prev_px = px;
                }
                &Chunk::Index(i) => {
                    img.push(hash[i as usize]);
                    prev_px = hash[i as usize];
                }
                &Chunk::Diff(DiffRGB(r, g, b)) => {
                    // for a Chunk::Diff(r,g,b), each of r, g, and b, is the difference from the previous pixel with a bias of 2.
                    //   0b00 => -2, 0b01 => -1, 0b10 => 0, 0b11 => 1
                    //   alpha is unchanged from prev pixel.
                    //   values wrap around at the u8 limit.
                    let cr = (prev_px.0).wrapping_add(r).wrapping_sub(2);
                    let cg = (prev_px.1).wrapping_add(g).wrapping_sub(2);
                    let cb = (prev_px.2).wrapping_add(b).wrapping_sub(2);
                    img.push(PixelRGBA(cr, cg, cb, prev_px.3));
                    hash[(cr as usize * 3
                        + cg as usize * 5
                        + cb as usize * 7
                        + prev_px.3 as usize * 11)
                        % 64] = PixelRGBA(cr, cg, cb, prev_px.3);
                    prev_px = PixelRGBA(cr, cg, cb, prev_px.3);
                }
                &Chunk::Luma(Luma(dg, dr_dg, db_dg)) => {
                    // for a Chunk::Luma(g, dr_dg, db-dg),
                    //  g is used to indicate the general direction of change and is encoded in 6 bits.
                    //  the red and blue channels (dr and db) base their diffs off of the green channel difference
                    //      dr_dg = (cur_px.r - prev_px.r) - (cur_px.g - prev_px.g)
                    //      dr_dg = cur_px.r - prev_px.r - g
                    //      dr_dg + g = cur_px.r - prev_px.r
                    //      dr_dg + g + prev_px.r = cur_px.r

                    //      db_dg = (cur_px.b - prev_px.b) - (cur_px.g - prev_px.g)
                    //      dr_dg = cur_px.b - prev_px.b - g
                    //      dr_dg + g = cur_px.b - prev_px.b
                    //      dr_dg + g + prev_px.b = cur_px.b
                    //  values are stored as unsigned integers with a bias of 32 for green channel and 8 for the red and blue channels.
                    //  values wrap around at the u8 limit.
                    //  alpha is unchanged from prev pixel.
                    let cr = (prev_px.0)
                        .wrapping_add(dr_dg)
                        .wrapping_add(dg)
                        .wrapping_sub(40);
                    let cg = (prev_px.1).wrapping_add(dg).wrapping_sub(32);
                    let cb = (prev_px.2)
                        .wrapping_add(db_dg)
                        .wrapping_add(dg)
                        .wrapping_sub(40);
                    img.push(PixelRGBA(cr, cg, cb, prev_px.3));
                    hash[(cr as usize * 3
                        + cg as usize * 5
                        + cb as usize * 7
                        + prev_px.3 as usize * 11)
                        % 64] = PixelRGBA(cr, cg, cb, prev_px.3);
                    prev_px = PixelRGBA(cr, cg, cb, prev_px.3);
                }
                &Chunk::Run(n) => {
                    // for a Chunk::Run(n), n is the number of exact copies of the previous pixel to make.
                    //  n has a bias of -1, meaning n=0 => 1.
                    for _ in 0..n + 1 {
                        img.push(prev_px);
                    }
                }
            }
        }

        let mut res = Vec::with_capacity(height);
        let mut cur_px = 0;
        for _ in 0..height {
            let mut row = Vec::with_capacity(width);
            for _ in 0..width {
                row.push(img[cur_px]);
                cur_px += 1;
            }
            res.push(row);
        }
        res
    }

    pub fn from_rgba_mat(src: &Vec<Vec<PixelRGBA>>, width: usize, height: usize) -> QOIImage {
        let mut is_transparent = false;
        let mut prev_px = PixelRGBA(0, 0, 0, 255);
        let mut hash = [PixelRGBA(0, 0, 0, 0); 64];
        let src = src.iter().flatten();
        let mut data: Vec<Chunk> = Vec::new();
        data.push(Chunk::RGBA(prev_px));
        for cur_px in src {
            // determine channels
            if cur_px.3 != 255 {
                is_transparent = true;
            }

            // check if this is a run
            if (cur_px.0, cur_px.1, cur_px.2, cur_px.3)
                == (prev_px.0, prev_px.1, prev_px.2, prev_px.3)
            {
                if let Chunk::Run(i) = data.last_mut().unwrap() {
                    *i += 1;
                } else {
                    data.push(Chunk::Run(0));
                }
                continue;
            }

            // check if this is appropriately an index
            let tmp = hash[(cur_px.0 as usize * 3
                + cur_px.1 as usize * 5
                + cur_px.2 as usize * 7
                + cur_px.3 as usize * 11)
                % 64];
            if (cur_px.0, cur_px.1, cur_px.2, cur_px.3) == (tmp.0, tmp.1, tmp.2, tmp.3) {
                data.push(Chunk::Index(
                    ((cur_px.0 as usize * 3
                        + cur_px.1 as usize * 5
                        + cur_px.2 as usize * 7
                        + cur_px.3 as usize * 11)
                        % 64) as u8,
                ));
                prev_px = *cur_px;
                continue;
            }

            // does this pixel change opacity from the last one?
            if cur_px.3 == prev_px.3 {
                // no => DIFF, LUMA, RGB
                let dr = cur_px.0 as i32 - prev_px.0 as i32;
                let dg = cur_px.1 as i32 - prev_px.1 as i32;
                let db = cur_px.2 as i32 - prev_px.2 as i32;

                // check if this pixel could be a DIFF
                if (dr >= -2 && dr <= 1) && (dg >= -2 && dg <= 1) && (db >= -2 && db <= 1) {
                    data.push(Chunk::Diff(DiffRGB(
                        (dr + 2) as u8,
                        (dg + 2) as u8,
                        (db + 2) as u8,
                    )));
                    hash[(cur_px.0 as usize * 3
                        + cur_px.1 as usize * 5
                        + cur_px.2 as usize * 7
                        + cur_px.3 as usize * 11)
                        % 64] = *cur_px;
                    prev_px = *cur_px;
                    continue;
                }
                // check if this pixel could be a LUMA
                //  if (-32 <= dg <= 31) then the green channel qualifies
                //      if (-8 <= (dr - dg) <= 7) && (-8 <= (db - dg) <= 7) then the red and blue channels qualify
                if (dg >= -32 && dg <= 31)
                    && ((dr - dg) >= -8 && (dr - dg) <= 7)
                    && ((db - dg) >= -8 && (db - dg) <= 7)
                {
                    data.push(Chunk::Luma(Luma(
                        (dg + 32) as u8,
                        (dr - dg + 8) as u8,
                        (db - dg + 8) as u8,
                    )));
                    hash[(cur_px.0 as usize * 3
                        + cur_px.1 as usize * 5
                        + cur_px.2 as usize * 7
                        + cur_px.3 as usize * 11)
                        % 64] = *cur_px;
                    prev_px = *cur_px;
                    continue;
                }

                // otherwise it has to be an RGB
                data.push(Chunk::RGB(PixelRGB(cur_px.0, cur_px.1, cur_px.2)));
                hash[(cur_px.0 as usize * 3
                    + cur_px.1 as usize * 5
                    + cur_px.2 as usize * 7
                    + cur_px.3 as usize * 11)
                    % 64] = *cur_px;
                prev_px = *cur_px;
                continue;
            } else {
                //  yes => RGBA
                data.push(Chunk::RGBA(*cur_px));
                hash[(cur_px.0 as usize * 3
                    + cur_px.1 as usize * 5
                    + cur_px.2 as usize * 7
                    + cur_px.3 as usize * 11)
                    % 64] = *cur_px;
                prev_px = *cur_px;
                continue;
            }
        }

        data.remove(0);
        let mut channels = Channels::RGB;
        if is_transparent {
            channels = Channels::RGBA;
        }
        QOIImage {
            width: width as u32,
            height: height as u32,
            channels,
            color_space: ColorSpace::Linear,
            data,
        }
    }
}

#[derive(Copy, Clone)]
pub struct PixelRGBA(u8, u8, u8, u8);

#[derive(Copy, Clone)]
pub struct PixelRGB(u8, u8, u8);

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{BufReader, Read},
    };

    use super::*;

    #[test]
    fn parse_dice() {
        QOIImage::from_qoi_file(BufReader::new(File::open("files/dice.qoi").unwrap()).bytes())
            .unwrap();
    }

    #[test]
    fn deserialize_reserialize_dice() {
        let dice =
            QOIImage::from_qoi_file(BufReader::new(File::open("files/dice.qoi").unwrap()).bytes())
                .unwrap();
        std::fs::write("files/dice2.qoi", dice.serialize()).unwrap();

        let file1 = BufReader::new(File::open("files/dice.qoi").unwrap()).bytes();
        let file2 = BufReader::new(File::open("files/dice2.qoi").unwrap()).bytes();
        for (b1, b2) in file1.zip(file2) {
            assert!(b1.unwrap() == b2.unwrap());
        }
    }

    #[test]
    fn dice_output_rgba() {
        let dice =
            QOIImage::from_qoi_file(BufReader::new(File::open("files/dice.qoi").unwrap()).bytes())
                .unwrap();

        let img = &mut dice
            .to_rgba_mat()
            .iter()
            .flatten()
            .map(|x| vec![x.0, x.1, x.2, x.3])
            .flatten()
            .collect::<Vec<u8>>();
        std::fs::write("files/dice.rgba", img).unwrap();

        let file1 = BufReader::new(File::open("files/dice.rgba").unwrap()).bytes();
        let file2 = BufReader::new(File::open("files/dice2.rgba").unwrap()).bytes();
        for (b1, b2) in file1.zip(file2) {
            assert!(b1.unwrap() == b2.unwrap());
        }
    }

    #[test]
    fn testcard_rgba_to_qoi_back_to_rgba() {
        let mut img = BufReader::new(File::open("files/testcard_rgba.rgba").unwrap()).bytes();
        let width = 256;
        let height = 256;
        let mut imgdata = Vec::new();
        let mut buf = [0u8; 4];
        let mut i = 0;
        while let Some(Ok(byte)) = img.next() {
            buf[i] = byte;
            i += 1;
            if i == 4 {
                imgdata.push(PixelRGBA(buf[0], buf[1], buf[2], buf[3]));
                i = 0;
            }
        }
        let mut i = 0;
        let mut mat = Vec::with_capacity(height);
        for _ in 0..height {
            let mut row = Vec::with_capacity(width);
            for _ in 0..width {
                row.push(*imgdata.get(i).unwrap());
                i += 1;
            }
            mat.push(row);
        }

        let testcard = QOIImage::from_rgba_mat(&mat, width, height);

        let testcard = &mut testcard
            .to_rgba_mat()
            .iter()
            .flatten()
            .map(|x| vec![x.0, x.1, x.2, x.3])
            .flatten()
            .collect::<Vec<u8>>();
        std::fs::write("files/testcard_rgba_output.rgba", testcard).unwrap();

        let file1 = BufReader::new(File::open("files/testcard_rgba_output.rgba").unwrap()).bytes();
        let file2 = BufReader::new(File::open("files/testcard_rgba.rgba").unwrap()).bytes();
        for (b1, b2) in file1.zip(file2) {
            assert!(b1.unwrap() == b2.unwrap());
        }
    }
}
