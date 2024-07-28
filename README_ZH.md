# projson

projson 通过使用 ProjFs 将 json 映射为文件树。效果如下图所示：

![introduce](./docs/introduce.png)

## Usage

在运行之前，请先运行以下 powershell 指令以启用 ProjFs 功能（ProjFs 是 Windows10 上的可选特性）。 

```powershell
Enable-WindowsOptionalFeature -Online -FeatureName Client-ProjFS -NoRestart
```

启用 ProjFs 功能后，可以使用 `projson --help` 查看用法信息：

```
Usage: projson.exe --prj-path <Virtual root directory path> <--json-file <Json file path>|--json-text <Json text>>
Options:
  -f, --json-file <Json file path>              Specifies the JSON file to read
  -t, --json-text <Json text>                   Specifies the JSON text to read
  -p, --prj-path <Virtual root directory path>  Specifies the virtualization root directory path
  -h, --help                                    Print help
  -V, --version                                 Print version
```

从 JSON 映射为文件或目录时，JSON 的 key 为文件或目录名，value 的类型将决定该 key-value 被映射为文件或目录，规则如下：

- Object: 目录，Object 子 key-value 为目录子项；
- Array: 目录，array 中元素为目录子项；
- String: 文件，文件内容是字符串值；
- Number: 文件，文件内容为数值；
- Boolean: 文件，文件内容为 'true' 或 'false'；
- Null: 空文件。

## Example

从 JSON 文件读取内容，映射为文件树：

```powershell
projson.exe --json-file src.json --prj-path D:\dst
```
