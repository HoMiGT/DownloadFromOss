# DownloadFromOss
> ## 项目说明
> 自定义从OSS上下载文件
> ## 环境变量
> 需要配置如下环境变量或者在程序执行路径下创建.env文件
> ```Shell
> ALIYUN_KEY_ID=xxxxxxx
> ALIYUN_KEY_SECRET=yyyyyyy
> ALIYUN_ENDPOINT=https://oss-cn-shanghai.aliyuncs.com
> ALIYUN_BUCKET=wpwl-hd2-img-test
> DOWNLOAD_COUNT=10
> DEFAULT_CACHE_FILE=files.txt
> PREFIX=zzzzzz
> ```
> 说明如下:
> * ALIYUN_KEY_ID: 连接key
> * ALIYUN_KEY_SECRET: 连接secret
> * ALIYUN_ENDPOINT: 站点
> * ALIYUN_BUCKET: 分区
> * DOWNLOAD_COUNT: 一次性查询的个数
> * DEFAULT_CACHE_FILE: 缓存文件，可以不修改，默认是files.txt
> * PREFIX: 查询oss上文件的前缀
> ## 使用说明
> * DownloadFromOss --help
> ```
> Usage: DownloadFromOss.exe [COMMAND]
> 
> Commands:
>  query           查询
>  download        下载
>  query-download  查询并下载
>  help            Print this message or the help of the given subcommand(s)
>
> Options:
>   -h, --help     Print help
>   -V, --version  Print version
>
> ```
> * 子命令的帮助
> ```
> 查询
>
> Usage: DownloadFromOss.exe query [OPTIONS]
> 
> Options:
>   -t, --time-range <TIME_RANGE>            时间范围 昨天数据 "2024-07-03"，区间范围 "2024-03-03~2024-07-03"
>   -n, --night-or-day <NIGHT_OR_DAY>        黑夜还是白天 默认值 0， 0-表示一整天 1-表示白天 2-表示黑夜
>   -d, --device <DEVICE>                    设备
>   -o, --org-id <ORG_ID>                    公司ID
>   -m, --material-number <MATERIAL_NUMBER>  物料号
>   -f, --file-name <FILE_NAME>              文件名称 需要全称,指定该参数其他参数将不可生效
>   -s, --save-file <SAVE_FILE>              保存查询结果的文件路径
>   -h, --help                               Print help
> 
> ```
