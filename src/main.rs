use std::fmt::Debug;
use dotenv::dotenv;
use clap::{Parser, Subcommand};
use aliyun_oss_client::{file::Files};
use std::{env, fs};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use chrono::{Days, NaiveDate};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// 下载
    #[command(subcommand)]
    subcommand: Option<Command>,
}
#[derive(Subcommand, Debug)]
enum Command {
    /// 查询
    Query {
        /// 时间范围 昨天数据 "2024-07-03"，区间范围 "2024-03-03~2024-07-03"
        #[arg(short('t'), long)]
        time_range: Option<String>,

        /// 黑夜还是白天 默认值 0， 0-表示一整天 1-表示白天 2-表示黑夜
        #[arg(short('n'), long)]
        night_or_day: Option<i32>,

        /// 设备
        #[arg(short('d'), long)]
        device: Option<String>,

        /// 公司ID
        #[arg(short('o'), long)]
        org_id: Option<i32>,

        /// 物料号
        #[arg(short('m'), long)]
        material_number: Option<String>,

        /// 文件名称 需要全称,指定该参数其他参数将不可生效
        #[arg(short('f'), long)]
        file_name: Option<String>,

        /// 保存查询结果的文件路径
        #[arg(short('s'), long)]
        save_file: Option<String>,
    },
    /// 下载
    Download {
        /// 下载文件列表 支持文件列表或者文本文件
        /// `-f value1 -f value2`
        #[arg(short('f'), long)]
        download_file: Option<Vec<String>>,
        /// 保存文件夹路径
        #[arg(short('d'), long)]
        save_dir: String,
    },
    /// 查询并下载
    QueryDownload {
        /// 时间范围 当天数据 "2024-07-03"，区间范围 "2024-03-03~2024-07-03"
        #[arg(short('t'), long)]
        time_range: Option<String>,

        /// 黑夜还是白天 默认值 0， 0-表示一整天 1-表示白天 2-表示黑夜
        #[arg(short('n'), long)]
        night_or_day: Option<i32>,

        /// 设备
        #[arg(short('d'), long)]
        device: Option<String>,

        /// 公司ID
        #[arg(short('o'), long)]
        org_id: Option<i32>,

        /// 物料号
        #[arg(short('m'), long)]
        material_number: Option<String>,

        /// 文件名称 需要全称,指定该参数其他参数将不可生效
        #[arg(short('f'), long)]
        file_name: Option<String>,

        /// 保存文件夹路径
        #[arg(short('s'), long)]
        save_dir: String,
    },
}

fn get_env(name: &str) -> Option<String> {
    match env::var(name) {
        Ok(v) => Some(v),
        Err(e) => None,
    }
}

struct MyOss;

impl MyOss {
    async fn query(time_range: Option<String>, night_or_day: Option<i32>, device: Option<String>,
                   org_id: Option<i32>, material_number: Option<String>, file_name: Option<String>, save_file: Option<String>) {
        let mut query_objs: Vec<String> = Vec::new();
        let default_cache_file = get_env("DEFAULT_CACHE_FILE").unwrap();
        let mut download_count = get_env("DOWNLOAD_COUNT").unwrap().parse::<i32>().unwrap();
        let prefix = get_env("PREFIX").unwrap_or("".to_string());
        match file_name {
            Some(v) => {
                query_objs.push(v);
            }
            None => {
                let mut date_vec: Vec<String> = Vec::new();
                let mut conditions: Vec<String> = Vec::new();
                match time_range {
                    Some(v) => {
                        let tmp = v.split("~").map(|x| x.to_string()).collect::<Vec<String>>();
                        if tmp.len() > 1 {
                            let start_date = NaiveDate::parse_from_str(tmp[0].as_str(), "%Y-%m-%d").unwrap();
                            let end_date = NaiveDate::parse_from_str(tmp[1].as_str(), "%Y-%m-%d").unwrap();
                            let mut current_date = start_date;
                            while current_date <= end_date {
                                date_vec.push(current_date.format("%Y-%m-%d").to_string());
                                current_date = current_date.checked_add_days(Days::new(1)).unwrap();
                            }
                        } else {
                            date_vec.push(tmp[0].to_string());
                        }
                    }
                    _ => {}
                }
                match night_or_day {
                    Some(v) => { conditions.push(v.to_string()) }
                    _ => {}
                }
                match device {
                    Some(v) => { conditions.push(v) }
                    _ => {}
                }
                match org_id {
                    Some(v) => { conditions.push(v.to_string()) }
                    _ => {}
                }
                match material_number {
                    Some(v) => { conditions.push(v) }
                    _ => {}
                }
                let condition_string = conditions.join("/");
                for date in date_vec {
                    if condition_string.is_empty() {
                        if prefix.is_empty() {
                            let tmp = format!("{}", &date);
                            query_objs.push(tmp);
                        } else {
                            let tmp = format!("{}/{}", prefix, &date);
                            query_objs.push(tmp);
                        }
                    } else {
                        if prefix.is_empty() {
                            let tmp = format!("{}/{}", &date, &condition_string);
                            query_objs.push(tmp);
                        } else {
                            let tmp = format!("{}/{}/{}", prefix, &date, &condition_string);
                            query_objs.push(tmp);
                        }
                    }
                }
            }
        }

        let mut save_file_string = "./".to_string();
        let mut save_file_path = Path::new(save_file_string.as_str());
        match save_file {
            Some(v) => {
                save_file_string = v.clone();
                save_file_path = Path::new(save_file_string.as_str());
                if !save_file_path.exists() {
                    match fs::create_dir_all(save_file_path) {
                        Ok(_) => println!("File path created successfully, {:?}", save_file_path),
                        Err(e) => eprintln!("Exception in creating non-existent file path, Error: {:#?}", e)
                    }
                }
            }
            _ => {}
        }
        let save_file_list_path_buf = save_file_path.join(default_cache_file.as_str());
        let save_file_list_path = save_file_list_path_buf.as_path();
        let mut file = fs::File::create(save_file_list_path).unwrap();

        match aliyun_oss_client::Client::from_env() {
            Ok(client) => {
                for obj in query_objs {
                    let query = [("max-keys".into(), download_count.to_string().into()), ("prefix".into(), obj.into())];
                    match client.get_object_list(query).await {
                        Ok(obj_list) => {
                            let obj_vec = obj_list.to_vec();
                            for obj in obj_vec {
                                let obj_path = obj.path();
                                file.write_fmt(format_args!("{}\n", &obj_path.to_string())).unwrap();
                            }
                        }
                        Err(e) => {
                            eprintln!("Client get object list error: {:?}", e);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Connect ALiYun Server Error: {:#?}", e);
            }
        };
    }

    async fn download(download_files: Option<Vec<String>>, save_dir: &str) {
        let default_cache_file = get_env("DEFAULT_CACHE_FILE").unwrap();
        match aliyun_oss_client::Client::from_env() {
            Ok(client) => {
                let mut download_file_list: Vec<String> = Vec::new();
                match download_files {
                    Some(download_files) => {
                        for download_file in download_files.iter() {
                            if download_file.ends_with(".txt") {
                                match fs::File::open(download_file) {
                                    Ok(file) => {
                                        let reader = BufReader::new(file);
                                        let content = reader.lines().map(|line| line.unwrap()).collect::<Vec<String>>();
                                        for line in content {
                                            download_file_list.push(line);
                                        }
                                    }
                                    Err(_) => {}
                                }
                            } else {
                                download_file_list.push(download_file.clone());
                            }
                        }
                    }
                    None => {
                        match fs::File::open(default_cache_file.as_str()) {
                            Ok(file) => {
                                let reader = BufReader::new(file);
                                let content = reader.lines().map(|line| line.unwrap().clone()).collect::<Vec<String>>();
                                for line in content {
                                    println!("line: {}", &line);
                                    download_file_list.push(line);
                                }
                            }
                            Err(_) => {
                                eprintln!("Please check if the local files.txt exists or enter the file path on OSS");
                            }
                        }
                    }
                }
                for download_file in download_file_list {
                    let download_file_path = Path::new(download_file.as_str());
                    let download_file_name = download_file_path.file_name().unwrap().to_str().unwrap();
                    let mut save_file_path_buf = PathBuf::new();
                    save_file_path_buf.push(save_dir);
                    save_file_path_buf.push(download_file_name);
                    let save_file_path = save_file_path_buf.as_path();
                    match client.get_object(download_file_path.to_str().unwrap().to_string(), ..).await {
                        Ok(v) => {
                            match fs::File::create(save_file_path) {
                                Ok(mut file) => {
                                    file.write(v.as_slice()).unwrap();
                                }
                                Err(_) => {}
                            }
                        }
                        Err(_) => {}
                    }
                }
            }
            Err(e) => {
                eprintln!("Connect ALiYun Server Error: {:#?}", e);
            }
        };
    }

    async fn query_download(time_range: Option<String>, night_or_day: Option<i32>, device: Option<String>,
                            org_id: Option<i32>, material_number: Option<String>, file_name: Option<String>, save_dir: &str) {
        let mut query_objs: Vec<String> = Vec::new();
        let mut download_count = get_env("DOWNLOAD_COUNT").unwrap().parse::<i32>().unwrap();
        let prefix = get_env("PREFIX").unwrap_or("".to_string());
        match file_name {
            Some(v) => {
                query_objs.push(v);
            }
            None => {
                let mut date_vec: Vec<String> = Vec::new();
                let mut conditions: Vec<String> = Vec::new();
                match time_range {
                    Some(v) => {
                        let tmp = v.split("~").map(|x| x.to_string()).collect::<Vec<String>>();
                        if tmp.len() > 1 {
                            let start_date = NaiveDate::parse_from_str(tmp[0].as_str(), "%Y-%m-%d").unwrap();
                            let end_date = NaiveDate::parse_from_str(tmp[1].as_str(), "%Y-%m-%d").unwrap();
                            let mut current_date = start_date;
                            while current_date <= end_date {
                                date_vec.push(current_date.format("%Y-%m-%d").to_string());
                                current_date = current_date.checked_add_days(Days::new(1)).unwrap();
                            }
                        } else {
                            date_vec.push(tmp[0].to_string());
                        }
                    }
                    _ => {}
                }
                match night_or_day {
                    Some(v) => { conditions.push(v.to_string()) }
                    _ => {}
                }
                match device {
                    Some(v) => { conditions.push(v) }
                    _ => {}
                }
                match org_id {
                    Some(v) => { conditions.push(v.to_string()) }
                    _ => {}
                }
                match material_number {
                    Some(v) => { conditions.push(v) }
                    _ => {}
                }
                let condition_string = conditions.join("/");
                for date in date_vec {
                    if condition_string.is_empty() {
                        if prefix.is_empty() {
                            let tmp = format!("{}", &date);
                            query_objs.push(tmp);
                        } else {
                            let tmp = format!("{}/{}", prefix, &date);
                            query_objs.push(tmp);
                        }
                    } else {
                        if prefix.is_empty() {
                            let tmp = format!("{}/{}", &date, &condition_string);
                            query_objs.push(tmp);
                        } else {
                            let tmp = format!("{}/{}/{}", prefix, &date, &condition_string);
                            query_objs.push(tmp);
                        }
                    }
                }
            }
        }
        match aliyun_oss_client::Client::from_env() {
            Ok(client) => {
                for obj in query_objs {
                    let query = [("max-keys".into(), download_count.to_string().into()), ("prefix".into(), obj.into())];
                    match client.get_object_list(query).await {
                        Ok(obj_list) => {
                            let obj_vec = obj_list.to_vec();
                            for obj in obj_vec {
                                let obj_path = obj.path();
                                let obj_path_str = obj_path.to_string();
                                let download_file_path = Path::new(obj_path_str.as_str());
                                let download_file_name = download_file_path.file_name().unwrap().to_str().unwrap();
                                let mut save_file_path_buf = PathBuf::new();
                                save_file_path_buf.push(save_dir);
                                save_file_path_buf.push(download_file_name);
                                let save_file_path = save_file_path_buf.as_path();
                                match client.get_object(download_file_path.to_str().unwrap().to_string(), ..).await {
                                    Ok(v) => {
                                        match fs::File::create(save_file_path) {
                                            Ok(mut file) => {
                                                file.write(v.as_slice()).unwrap();
                                            }
                                            Err(_) => {}
                                        }
                                    }
                                    Err(_) => {}
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Client get object list error: {:?}", e);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Connect ALiYun Server Error: {:#?}", e);
            }
        };
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let args = Args::parse();
    match args.subcommand {
        Some(command) => {
            match command {
                Command::Query {
                    time_range,
                    night_or_day,
                    device,
                    org_id,
                    material_number,
                    file_name,
                    save_file
                } => {
                    MyOss::query(time_range, night_or_day, device, org_id, material_number, file_name, save_file).await;
                }
                Command::Download {
                    download_file,
                    save_dir
                } => {
                    println!("download_file: {:?}", download_file);
                    println!("save_dir: {:?}", save_dir);
                    MyOss::download(download_file, save_dir.as_str()).await;
                }
                Command::QueryDownload {
                    time_range,
                    night_or_day,
                    device,
                    org_id,
                    material_number,
                    file_name,
                    save_dir
                } => {
                    MyOss::query_download(time_range, night_or_day, device, org_id, material_number, file_name, save_dir.as_str()).await;
                }
            }
        }
        None => {}
    }
}

