#![feature(path_ext)]
#![feature(convert)]
#![feature(deque_extras)]
extern crate lodepng;
extern crate byteorder;
extern crate alsa;

use std::cmp::Ordering;
use std::io::{SeekFrom, Seek};
use std::path::{Path};
use std::fs::{File, PathExt};
use byteorder::{ReadBytesExt, LittleEndian};

mod algos;
mod dsp;

use dsp::DSPBlockConvertSpec;

use algos::SignalMap;
use algos::mcguire_smde;

fn main() {
    /*
    let fpath: &Path = Path::new("/home/kmcguire/Projects/radiowork/usbstore/test147.840");
    
    let fsize: usize = match fpath.metadata() {
        Ok(md) => md.len() as usize,
        Err(_) => panic!("Could not get file length."),
    };
    
    let mut fp = File::open(fpath).unwrap();

    let mut min = std::f64::MAX;
    let mut max = std::f64::MIN;
    let mut sum = 0f64;

    for x in 0..fsize / 4 {
        let sample: f64 = fp.read_f32::<LittleEndian>().unwrap() as f64;
        
        if sample < min {
            min = sample;
        }
        
        if sample > max {
            max = sample;
        }
        
        sum += sample;
    }
    
    println!("min:{} max:{} avg:{}", min, max, sum / ((fsize as f64) / 4f64));
    
    if true { return; }    
    */
    //other();

    //if true {
    //    return;
    //}

    let fpath: &Path = Path::new("/home/kmcguire/Projects/radiowork/usbstore/recording01");
    
    let fsize: usize = match fpath.metadata() {
        Ok(md) => md.len() as usize,
        Err(_) => panic!("Could not get file length."),
    };
    
    let mut fp = File::open(fpath).unwrap();
    
    let mut lpf = dsp::DSPBlockLowPass::new(1f64);
    let mut lpf_split = dsp::DSPBlockSplitter::new();
    let mut audio = dsp::DSPBlockSinkAudio::new(16000);
    let mut conv = dsp::DSPBlockConvert::new(25);
    let mut fm = dsp::DSPBlockFMDecode::new();
    let mut fm_split = dsp::DSPBlockSplitter::new();
    
    let mut bf0: dsp::DSPBlockSinkFile<dsp::Complex<f64>> = dsp::DSPBlockSinkFile::new(Path::new("/home/kmcguire/Projects/radiowork/usbstore/lpfout"));
    let mut bf1: dsp::DSPBlockSinkFile<f64> = dsp::DSPBlockSinkFile::new(Path::new("/home/kmcguire/Projects/radiowork/usbstore/fmout"));
    
    let (chan_begin, lpf_in) = dsp::dspchan_create();
    
    let (lpf_out, lpf_split_in) = dsp::dspchan_create();
    let (lpf_split_out0, fm_in) = dsp::dspchan_create();
    let (lpf_split_out1, bf0_in) = dsp::dspchan_create();    
    
    let (fm_out, fm_split_in) = dsp::dspchan_create();
    let (fm_split_out0, conv_in) = dsp::dspchan_create();
    let (fm_split_out1, bf1_in) = dsp::dspchan_create();
    
    let (conv_out, audio_in) = dsp::dspchan_create();
    
    
    lpf.set_input(lpf_in);
    lpf.set_output(lpf_out);
    
    lpf_split.set_input(lpf_split_in);
    lpf_split.add_output(lpf_split_out0);
    lpf_split.add_output(lpf_split_out1);
    
    bf0.set_input(bf0_in);
    bf1.set_input(bf1_in);
    
    fm.set_input(fm_in);
    fm.set_output(fm_out);
    
    fm_split.set_input(fm_split_in);
    fm_split.add_output(fm_split_out0);
    fm_split.add_output(fm_split_out1);
    
    conv.set_input(conv_in);
    conv.set_output(conv_out);
    
    audio.set_input(audio_in);

    let mut sent = 0;    
    
    /*for x in 0..fsize / 8 / 10 {
        fp.seek(SeekFrom::Start((x * 10 * 8) as u64)).unwrap();
        
        let i = fp.read_f32::<LittleEndian>().unwrap() as f64;
        let q = fp.read_f32::<LittleEndian>().unwrap() as f64;
    */
    
    for x in 0..1024 * 1024 * 100 {
        let i = 30000.0 * (x as f64 * 0.1 * 3.141).sin();
        let q = 0.0 * (x as f64 * 0.1 * 3.141).sin();
    
        chan_begin.send(dsp::Complex { i: i, q: q });
        sent += 1;
        if sent > 1024 * 1024 * 4 {
            use dsp::DSPBlockSinkFileSpec;
            
            println!("block A processing");
            lpf.tick_uoi();
            lpf_split.tick_uoi();
            bf0.tick_uoi();
            println!("block B processing");
            fm.tick_uoi();
            fm_split.tick_uoi();
            bf1.tick_uoi();
            println!("block C processing");
            conv.tick_uoi();
            println!("block D processing");
            audio.tick_uoi();
            println!("audio sent {}", audio.get_consumed());
            sent = 0;
            println!("sending data to block");
        }
    } 
    
    println!("done");
}


fn other() {
    let fpath: &Path = Path::new("/home/kmcguire/Projects/radiowork/usbstore/spectrum");
    
    let fsize: usize = match fpath.metadata() {
        Ok(md) => md.len() as usize,
        Err(_) => panic!("Could not get file length."),
    };
    
    let mut fp = File::open(fpath).unwrap();
    
    let fw: usize = 1024;
    let fh: usize = fsize / 4 / fw;

    let factorw: usize = 1;
    let factorh: usize = 4;    
    
    let aw: usize = fw / factorw;
    let ah: usize = fh / factorh;
    
    let mut abuffer: Vec<f64> = Vec::with_capacity(aw * ah * 2); 
    let mut pbuffer: Vec<u8> = Vec::with_capacity(aw * ah * 4);
    
    for _ in 0..aw * ah {
        abuffer.push(0.0);
        abuffer.push(0.0);
        pbuffer.push(0);
        pbuffer.push(0);
        pbuffer.push(0);
        pbuffer.push(0);
    }
    
    let mut sample_max: f64 = std::f64::MIN;
    let mut sample_min: f64 = std::f64::MAX;
    
    for col in 0..ah {
        println!("working col: {}/{}", col, ah); 
        for row in 0..aw {
            let rowrng = (row * factorw, row * factorw + factorw);
            let colrng = (col * factorh, col * factorh + factorh);
            for fcol in (colrng.0)..(colrng.1) {
                fp.seek(SeekFrom::Start((fcol * fw + rowrng.0) as u64 * 4)).unwrap();
                for _ in 0..factorw {
                    let sample: f64 = fp.read_f32::<LittleEndian>().unwrap() as f64;      
                    abuffer[(row + col * aw) * 2 + 0] += sample as f64;
                    abuffer[(row + col * aw) * 2 + 1] += 1.0f64;
                }
            }
        }        
    }
    
    /*
    let mut _tmp: Vec<f64> = Vec::with_capacity(aw*ah);
    
    // Produce the average per each output unit.
    for row in 0..ah {
        for col in 0..aw {
            _tmp.push(abuffer[(row * aw + col) * 2 + 0] / abuffer[(row * aw + col) * 2 + 1]);
        }
    }    

    let mut amap = SignalMap {
        v:      _tmp,
        w:      aw,
        h:      ah,
    };    
     
    for _ in 0..1 {
        amap = mcguire_smde::multi_all(&amap);
        amap.normalize();
    }
    
    let mut __tmp: Vec<u8> = Vec::with_capacity(amap.w * amap.h);
    
    for x in 0..amap.v.len() {
        __tmp.push((amap.v[x] * 255f64) as u8); // R
        __tmp.push((amap.v[x] * 255f64) as u8); // G
        __tmp.push((amap.v[x] * 255f64) as u8); // B
        __tmp.push(255);
    }
    
    lodepng::encode_file(
        "ana.png", __tmp.as_slice(),
        amap.w, amap.h,
        lodepng::ColorType::LCT_RGBA, 8
    ).unwrap();    
    */
    
    let space = (3.1415 * 2.0 / 3.0) as f64;
    
    let mut rowdata: Vec<f64> = Vec::with_capacity(aw);
    
    for i in 0..aw {
        rowdata.push(0.0f64);
    }    

    let mut gmin: f64 = std::f64::MAX;
    let mut gmax: f64 = std::f64::MIN;
    
    println!("gmin:{} gmax:{}", gmin, gmax);  
    
    for i in 0..aw * ah {
        let f = abuffer[i * 2 + 0] / abuffer[i * 2 + 1];
        if f < gmin {
            gmin = f;
        }
        
        if f > gmax {
            gmax = f;
        }
    }
    
    for col in 0..ah {
        let colpos = col * aw;
        
        let mut min: f64 = std::f64::MAX;
        let mut max: f64 = std::f64::MIN;
        
        for row in 0..aw {
            rowdata[row] = abuffer[(colpos + row) * 2 + 0] / abuffer[(colpos + row) * 2 + 1];
            
            if rowdata[row] > max {
                max = rowdata[row];
            }
            
            if rowdata[row] < min {
                min = rowdata[row];
            }
        }
            
        for row in 0..aw {
            // Large differences get a dark value so invert the normalization.
            let mut samplenorm = 1.0 - (rowdata[row] - gmin) / (gmax - gmin);
            
            //let i: f64 = samplenorm * 3.1415 * 2.0;
            
            //let r = (i.sin() * 255f64) as u8;
            //let g = ((i + space).sin() * 255f64) as u8;
            //let b = ((i + space * 2f64).sin() * 255f64) as u8;
            
            let r = samplenorm * 255f64;
            let g = samplenorm * 255f64;
            let b = samplenorm * 255f64; 
            
            pbuffer[(colpos + row) * 4 + 0] = r as u8;
            pbuffer[(colpos + row) * 4 + 1] = g as u8;
            pbuffer[(colpos + row) * 4 + 2] = b as u8;
            pbuffer[(colpos + row) * 4 + 3] = 255 as u8;
        }
    }    

    println!("aw:{} ah:{} pbuffer.len():{}", aw, ah, pbuffer.len());
    lodepng::encode_file(
        "out.png", pbuffer.as_slice(),
        aw, ah,
        lodepng::ColorType::LCT_RGBA, 8
    ).unwrap();
}
