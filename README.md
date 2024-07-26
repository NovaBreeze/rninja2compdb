# rninja2compdb

A simple binary program to generate the compile_command.json repository from ninja.

## Build

```bash
git clone https://github.com/NovaBreeze/rninja2compdb.git
cd rninja2compdb
cargo build --release
install -m 755 target/release/rninja2compdb /usr/local/bin/
```
## Usage

The `input file` can be a `ninja` file or a `compiled database json file`. Different types of input files will have different behaviors.

**From ninja:**

Extract the compilation parameters for clang/clang++ from Android's ninja files to generate the repository file compile_commands.json .
After completion, it will prompt `Done` .

```bash
> rninja2compdb -i path/to/ninja -r /android/source/root/path/for/ninja
Done
```

Android's ninja files are usually very large, and despite only taking the compilation parameters of the clang/clang++ in them, they are still up to 100 megabytes, which is still difficult for clangd parsing.
Therefore, you can specify the `-P` parameter to extract only the modules you need. The argument to `-P` should be a relative path, and an Android source path relative to `-r | --root`

```bash
> rninja2compdb -i path/to/ninja -r /android/source/root/path/for/ninja -P path/to/module1 -P path/to/module2 -P ...
```

**From json:**

The json file is usually used as an input file when you need to trim the compile_commands.json file.
Therefore, you should need to enter the `-P` parameter, otherwise nothing will happen.

```bash
> rninja2compdb -i path/to/compdb.json -P path/to/module1 -P ...
```

**Set output directory:**

Executing the above command will generate compile_commands.json in the current working path. If you need to specify the save path, specify the `-o` parameter.

```bash
> rninja2compdb -i path/to/ninja -r /android/source/root/path/for/ninja -o path/to/save
```

**Set output filename:**

The default output file name is compile_commands.json, modified with the `-f` parameter.

```bash
> rninja2compdb -i path/to/ninja -r /android/source/root/path/for/ninja -f compile_commands_alias.json
```

**Use config file:**

All parameters required by the command can be obtained from the configuration file. Use the `-c` parameter to specify the configuration file path.
Specify -c - to obtain a configuration template based on the current parameters. Such as

```bash
> rninja2compdb -i path/to/ninja -r /android/source/root/path/for/ninja -P path/to/module1 -P path/to/module -f compile_commands_alias.json -c -

# You will get a template.json in the current directory
> cat template.json
{
  "input": "path/to/ninja",
  "root": "/android/source/root/path/for/ninja",
  "output": ".",
  "filename": "compile_commands_alias.json",
  "pretty": true,
  "patterns": [
    "path/to/module1",
    "path/to/module"
  ]
}

# You can then use `-c template.json` directly to achieve the same effect as the previous command.
> rninja2compdb -c template.json
```

**Help:**

Run `rninja2compdb --help` to get the arguments.

```bash
> rninja2compdb --help

A simple binary program to generate the compile_command.json repository from ninja.

Usage: rninja2compdb [OPTIONS]

Options:
  -i, --input <FILE>       The path to the input file. If it's a .ninja file, it will be parsed to generate a clangd tag repository; if it's a .json repository file, it will extract entries that match the patterns parameter, or do nothing if no patterns are specified
  -r, --root <DIR>         Android root directory. Please use absolute path
  -o, --output <DIR>       Output directory [default: .]
  -f, --filename <NAME>    Filename to process [default: compile_commands.json]
  -p, --on-pretty          Pretty-print output
  -P, --pattern <PATTERN>  Patterns to match
  -c, --config <FILE>      Parameter configuration file, if "-" is given, a template file is generated based on the current parameters
  -h, --help               Print help
  -V, --version            Print version

```

