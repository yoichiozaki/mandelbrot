extern crate num;
use num::Complex;

use std::str::FromStr;

/// Parse the string 's' as a coordinate pair, like '"400x600"', or '"1.0,5.0"'.
///
/// Specifically, 's' should have the form <left><sep><right>, where <sep> is
/// the character given by the 'separator' argument, and <left> and <right> are both
/// strings that can be parsed by 'T::from_str'
///

/// If 's' has the proper form, return 'Some<(x, y)>'. If it doesn't parse
/// correctly, return 'None'.

fn parse_pair<T:FromStr>(s: &str, separator: char) -> Option<(T, T)> {
    match s.find(separator) {
        None => None,
        Some(index) => {
            match (T::from_str(&s[..index]), T::from_str(&s[index+1..])) {
                (Ok(l), Ok(r)) => Some((l, r)),
                _ => None
            }
        }
    }
}

#[test]
fn test_parse_pair() {
    assert_eq!(parse_pair::<i32>("", ','), None);
    assert_eq!(parse_pair::<i32>("10,", ','), None);
    assert_eq!(parse_pair::<i32>(",10", ','), None);
    assert_eq!(parse_pair::<i32>("10,20", ','), Some((10, 20)));
    assert_eq!(parse_pair::<i32>("10, 20xy", ','), None);
    assert_eq!(parse_pair::<f64>("0.5x", 'x'), None);
    assert_eq!(parse_pair::<f64>("0.5x1.5", 'x'), Some((0.5, 1.5)));
}

/// Parse a pair of floating-point numbers separated by a comma as a complex number.
fn parse_complex(s: &str) -> Option<Complex<f64>> {
    match parse_pair(s, ',') {
        Some((re, im)) => Some(Complex{re, im}),
        None => None
    }
}

#[test]
fn test_parse_complex() {
    assert_eq!(parse_complex("1.25,-0.0625"), Some(Complex{re: 1.25, im:-0.0625}));
    assert_eq!(parse_complex(",-0.0625"), None)
}

/// Given the row and column of a pixel in the output image, return the
/// corresponding point on the complex plane.
///
/// 'bounds' is a pair giving the width and height of the image in pixels.
/// 'pixel' is a (column, row) pair indicating a particular pixel in that image.
/// the 'upper_left' and 'lower_right' parameters are points on the complex
/// plane designating the area our image covers.
fn pixel_to_point(
    bounds: (usize, usize),
    pixel: (usize, usize),
    upper_left: Complex<f64>,
    lower_right: Complex<f64>)
        -> Complex<f64> {
    let (width, height) = (
        lower_right.re - upper_left.re,
        upper_left.im - lower_right.im);
    Complex {
        re: upper_left.re + pixel.0 as f64 * width / bounds.0 as f64,
        im: upper_left.im - pixel.1 as f64 * height / bounds.1 as f64
        // Why subtraction here? pixel.1 increases as we go down,
        // but the imaginary component increases as we go up.
    }
}

#[test]
fn test_pixel_to_point() {
    assert_eq!(pixel_to_point(
        (100, 100),
        (25, 75),
        Complex {re: -1.0, im: 1.0},
        Complex {re: 1.0, im: -1.0}
    ), Complex {re: -0.5, im: -0.5})
}

/// Try to determine if 'c' is in the Mandelbrot set, using at most 'limit' iterations to decide.
///
/// If 'c' is not a member, return 'Some(i)', where 'i' is the number of
/// iterations it took for 'c' to leave the circle of radius two centered on the
/// origin. If 'c' seems to be a member (more precisely, if we reached the
/// iteration limit without being able to prove that 'c' is not a member),
/// return 'None'.
fn escape_time(c: Complex<f64>, limit: u32) -> Option<u32> {
    let mut z = Complex{re: 0.0, im: 0.0};
    for i in 0..limit {
        z = z * z + c;
        if z.norm_sqr() > 4.0 {
            return Some(i);
        }
    }
    None
}

/// Render a rectangle of the Mandelbrot set into a buffer of pixels.
///
/// The 'bounds' argument gives the width and height of the buffer 'pixels',
/// which holds one gray-scale pixel per byte. The 'upper_left' and 'lower_right'
/// arguments specify points on the complex plane corresponding to the upper-left and
/// lower-right corners of the pixel buffer.
fn render(
    pixels: &mut [u8],
    bounds: (usize, usize),
    upper_left: Complex<f64>,
    lower_right: Complex<f64>)
{
    assert_eq!(pixels.len(),  bounds.0 * bounds.1);

    for row in 0 .. bounds.1 {
        for column in 0 .. bounds.0 {
            let point = pixel_to_point(bounds, (column, row), upper_left, lower_right);
            pixels[row * bounds.0 + column] =
                match escape_time(point, 255) {
                    None => 0, // If the point is included in the Mandelbrot set, paint it as black(0).
                    Some(count) => 255 - count as u8
                };
        }
    }
}

extern crate image;

use image::ColorType;
use image::png::PNGEncoder;
use std::fs::File;

/// Write the buffer 'pixels', whose dimensions are given by
/// 'bounds', to the file name 'filename'.
fn write_image(filename: &str, pixels: &[u8], bounds: (usize, usize))
    -> Result<(), std::io::Error> {
    // Open the output file.
    let output = File::create(filename)?;

    // let output = match File::create(filename) {
    //     Ok(f) => { f },
    //     Err(e) => { return Err(e); }
    // };

    // create a png encoder.
    let encoder = PNGEncoder::new(output);
    encoder.encode(&pixels, bounds.0 as u32, bounds.1 as u32, ColorType::Gray(8))?;

    Ok(()) // If write_image() work successfully, it returns () (unit).
}

extern crate crossbeam;
use std::io::Write;

fn main() {
    println!("Hello, Mandelbrot!");
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 5 {
        writeln!(
            std::io::stderr(),
            "Usage: mandelbrot FILE PIXELS UPPER-RIGHT LOWER-RIGHT"
        ).unwrap();
        writeln!(
            std::io::stderr(),
            "Example: {} mandel.png 1000x750 -1.20,0.35 -1,0.20",
            args[0]
        ).unwrap();
        std::process::exit(1);
    }
    let bounds = parse_pair(&args[2], 'x')
        .expect("error parsing image dimensions.");
    let upper_left = parse_complex(&args[3])
        .expect("error parsing upper left corner point on complex plane.");
    let lower_right = parse_complex(&args[4])
        .expect("error parsing lower right corner point on complex plane.");
    let mut pixels = vec![0; bounds.0 * bounds.1];

    // single thread version
    // // ここではpixelsがバッファベクタの所有者であることを維持したままバッファへの可変参照を「借用」している。
    // render(&mut pixels, bounds, upper_left, lower_right);

    // multi threads version
    let threads = 8;
    let rows_per_band = bounds.1 / threads + 1;

    {
        // chunk_mutメソッドは可変な重なり合わないバッファのスライを生成するイテレータを返す。
        let bands: Vec<&mut [u8]> = pixels.chunks_mut(rows_per_band * bounds.0).collect();

        // closure式：引数の型を宣言する必要がない。Rustコンパイラが引数の型と返り値の型を推論する。
        // crossbeam::scope(| <arguments> | { ...function body... } )
        // crossbeam::scope関数はクロージャ内の全てのスレッドが完了するのを待ってから終了する。
        crossbeam::scope(|spawner| {

            // into_iterメソッドがここの帯領域に対する排他的な所有権を各ループのボディに与える。これによって
            // 唯一そのスレッドだけがその帯領域に書き込むことができるようになることを保証する。
            for (i, band) in bands.into_iter().enumerate() {

                // 個々の帯領域を決定する
                let top = rows_per_band * i;
                let height = band.len() / bounds.0;
                let band_bounds = (bounds.0, height);
                let band_upper_left = pixel_to_point(
                    bounds,
                    (0, top),
                    upper_left,
                    lower_right
                );
                let band_lower_right = pixel_to_point(
                    bounds,
                    (bounds.0, top + height),
                    upper_left,
                    lower_right
                );

                // spawn = 生まれる; ここでクロージャ内で新しいスレッドを生成する。
                // spawn( move || { ...function body... } )
                // 引数なしのクロージャを実行する。'move'によってクロージャがその内部で利用する変数への所有権を
                // 取得することを表現している。ここでは'band'の所有権を取得していることになる。
                spawner.spawn(move || {
                    render(band, band_bounds, band_upper_left, band_lower_right)
                });
            }
        })
    }

    // write_imageはバッファベクタの内容を変更するはずがないので、不変な共有参照を渡している。
    write_image(&args[1], &pixels, bounds)
        .expect("error writing PNG file.");
}