# Write and run your first Semauri program

**Semauri is a programming language.** Its programs are written in plain-text files ending with `.suri`. You can create those files in any text editor, compile or check them with the `semauri` command, and run supported programs explicitly.

This guide assumes you have never used a terminal before. Choose the section for your operating system.

## 1. Open a terminal

| System | How to open the terminal |
| --- | --- |
| Windows 10/11 | Open **Start**, type **PowerShell**, and launch **Windows PowerShell** or **Terminal**. Ordinary user privileges are enough. |
| macOS | Press **Command + Space**, type **Terminal**, and press **Return**. |
| Linux | Open the applications menu and choose **Terminal**, or try **Ctrl + Alt + T** on common desktop environments. |

A terminal is a text interface that runs commands. Type the commands shown below **without** the `$` or `>` prompt decorations you may see in other tutorials. Press **Enter/Return** to execute each command.

## 2. Install Semauri

You do **not** need Rust or Cargo to use a published native release.

### Windows: PowerShell

```powershell
irm https://raw.githubusercontent.com/edujbarrios/semauri/main/install.ps1 | iex
```

Close and reopen PowerShell after installation so updated `PATH` settings take effect, then enter:

```powershell
semauri version
semauri help
```

### macOS and Linux: Terminal

```sh
curl -fsSL https://raw.githubusercontent.com/edujbarrios/semauri/main/install.sh | sh
```

Then check:

```sh
semauri version
semauri help
```

**Review installation scripts before running them** if you prefer: [install.sh](../install.sh) and [install.ps1](../install.ps1). The installers download platform-specific GitHub Release artifacts and verify the archive's checksum. You may instead [download and verify an archive manually](00_INSTALLATION.md#manual-download).

If `semauri` is not recognized, follow the [PATH troubleshooting instructions](00_INSTALLATION.md#troubleshooting) rather than reinstalling Rust.

## 3. Create a folder for your project

In **PowerShell**:

```powershell
cd $HOME
mkdir semauri-first
cd semauri-first
```

On **macOS or Linux**:

```sh
cd ~
mkdir semauri-first
cd semauri-first
```

The `cd` command changes directories; `mkdir` creates a directory. Your files now belong to a dedicated folder rather than being scattered through Downloads or your home directory.

## 4. Create a plain-text `.suri` file

On **Windows**, type `notepad hello.suri` in PowerShell. Accept the prompt to create the file if shown.

On **macOS or Linux**, type `nano hello.suri` if the `nano` editor is installed. Alternatively open the project folder in any editor (such as Visual Studio Code) and create a file named `hello.suri`.

Copy this complete Semauri program into the editor:

```sema
let answer be 6 times 7.
assert answer is equal to 42.
print "Hello from Semauri".
print answer.
```

Save the file. With `nano`, press **Ctrl+O**, **Enter**, then **Ctrl+X**. In Notepad, use **File → Save**, select **UTF-8** and ensure the filename is exactly `hello.suri`, **not** `hello.suri.txt`.

## 5. Check, inspect and run the program

The following commands work in PowerShell, macOS Terminal and Linux Terminal while you are inside `semauri-first`.

Check syntax and types **without running the program**:

```sh
semauri check hello.suri
```

Inspect the planned execution without printing:

```sh
semauri run --dry-run hello.suri
```

Authorize the console output and execute:

```sh
semauri run --allow console_write hello.suri
```

Expected output:

```text
Hello from Semauri
42
```

Semauri's effects are explicit: ordinary `print` requires `console_write` when run. A failed `assert` reports a source-located error rather than silently continuing.

## 6. Edit your code and try again

Change `42` to `43` in the assertion, save, and run it again. The assertion fails, which demonstrates that Semauri actually type-checks and executes the program's supported operations. Restore the value to `42`.

You can also format your code using canonical lowercase commands:

```sh
semauri fmt hello.suri --write
```

The formatter changes language keywords, not strings such as `"Hello from Semauri"`.

## 7. Continue learning

- [Language syntax](01_LANGUAGE.md) — variables, expressions, type checking, conditions and lists
- [Learn to Code in English](../learn_to_code/en/README.md) — 11 progressively harder lessons
- [Learn to Code in Spanish](../learn_to_code/es/README.md) — 11 lessons with the same English-language Semauri syntax
- [CLI and explicit capabilities](05_RUNTIME.md) — supported runtime effects and security boundaries
- [Build a web page or schema](03_DOMAINS.md) — examples using Semauri's compilation backends
- [Building Semauri from source](DEVELOPMENT.md) — for contributors who want to change the compiler

## Frequent beginner questions

**I see `command not found` or `semauri is not recognized`.** Open a new terminal, verify the installation and review the [PATH fixes](00_INSTALLATION.md#troubleshooting).

**My file is missing.** Verify you are in the right directory with `pwd` on macOS/Linux or `Get-Location` in PowerShell, then list files using `ls` or `dir`.

**My filename ends in `.txt`.** Enable file extensions in Windows File Explorer and rename `hello.suri.txt` to `hello.suri`.

**Do I have to use a particular editor?** No. A Semauri file is plain text. Notepad, nano, VS Code or another editor can create it.

**Does Semauri execute arbitrary Python or ML models?** No. The Semauri language currently executes selected authorized operations and compiles typed domain programs. Its ML domain represents typed workflow plans; real model training requires an execution backend.
