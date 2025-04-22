/*
 * This file is part of PROJECT.
 *
 * PROJECT is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * PROJECT is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with PROJECT.  If not, see <https://www.gnu.org/licenses/>.
 */

use serialport::{SerialPort};
use std::io::{self, Write};
use std::process;
use regex::Regex;
use binaryfile::{BinaryWriter};

pub struct SerialComm {
    pub port: Box<dyn SerialPort>,
    pub logpath: String,
    pub monitor: bool,
}

impl SerialComm {
    pub fn new(port_name: &str, speed: u32) -> Result<Self, io::Error> {
        let port = Box::new(serialport::new(port_name, speed).open_native()?);
        Ok(Self { 
            port: port,
            logpath: "".to_string(),
            monitor: true,
        })
    }

    pub fn set_monitoring(&mut self, monitor: bool) {
        self.monitor = monitor;
    }

    pub fn set_logpath(&mut self, logpath: &str) {
        self.logpath = logpath.to_string();
        let fw = BinaryWriter::new(&self.logpath);
    }

    pub fn write(&mut self, cmd: &str) {
        self.port.write_all(cmd.as_bytes()).unwrap();
    }

    /*
     * targetは正規表現のパターンを指定する。（単に単語でもOK）
     * 複数のパターンでヒットするような場合戻り値を見ることでどれにヒットしたのかわかる。
     */ 
    pub fn wait_for(&mut self, target: &str) -> String {
        //タイムアウトを60秒に設定
        self.port.set_timeout(std::time::Duration::from_secs(60)).unwrap();

        let mut linebuffer:Vec<u8> = Vec::new();
        let re = Regex::new(target).unwrap();

        loop {

            let ab = self.port.bytes_to_read().unwrap();
            if ab > 0 { // データが有るので読み込んで処理

                let mut buffer:Vec<u8> = vec![0; ab.try_into().unwrap()];
                self.port.read_exact(&mut buffer).unwrap();

                // モニタリングが有効なら画面に表示
                if self.monitor {
                    print!("{}", String::from_utf8_lossy(&buffer));
                    io::stdout().flush().unwrap();
                }

                // logpathが設定されていたらファイルに出力
                if self.logpath != "" {
                    let mut fw = BinaryWriter::open(&self.logpath);
                    fw.write(&buffer);
                }

                // ラインバッファを積み、パターンにヒットするかチェックする。
                // ヒットした場合ヒットした文字列を返す。
                linebuffer.extend(&buffer);
                let data = String::from_utf8_lossy(&linebuffer);
                if let Some(caps) = re.captures(&data) {
                    return caps[0].to_string();
                }

                // 改行があったらラインバッファをクリアする。
                if let Some(index) = buffer.iter().position(|&x| x == 0x0A || x == 0x0D ) {
                    //let linebuffer:Vec<u8> = linebuffer[index+1..].to_vec();
                    let mut linebuffer:Vec<u8> = Vec::new();
                    println!("buffer clear");
                }
            }
        }
    }
}


pub fn get_portlist() -> Vec<String> {
    let mut seriallist: Vec<String> = Vec::new();
    match serialport::available_ports() {
        Ok(ports) => {
            for port in ports {
                seriallist.push(port.port_name);
            }
        },
        Err(e) => println!("{:?}", e)
    }
    seriallist
}

pub fn get_port() -> String {
    let portlist = get_portlist();
    let mut port:String = "".to_string();
    if portlist.len() == 1 {
        port = format!("{}", &portlist[0]); 
    } else if portlist.len() == 0 {
        println!("serial is not alive!");
        process::exit(0);
    } else {
        println!("many serials");
        process::exit(0);
    }
    return port;
}


/*
fn main() {

    let port = serialobj::get_port();
    let mut sp = SerialComm::new(&port, 9600).unwrap();

    sp.set_logpath("log.txt");

    sp.wait_for("login:");
    sp.write("admin\n");
    sp.wait_for("Password:");
    sp.write("admin\n");

    let ret = sp.wait_for("(WA2021>|Login incorrect)");
    
    if ret == "Login incorrect" {
        println!("INCORRECT !!!!!");
    } else {
        sp.write("terminal length 0\n");
        sp.wait_for("WA2021>");
        sp.write("show run\n");
        sp.wait_for("WA2021>");
    }

    sp.set_logpath("");
}
*/
