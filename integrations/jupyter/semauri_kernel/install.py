"""Install a Semauri kernel spec into the current user's Jupyter data dir."""
import argparse
import json
from pathlib import Path
import sys
import tempfile

from jupyter_client.kernelspec import KernelSpecManager


def main(argv=None):
    parser = argparse.ArgumentParser(description="Install the Semauri Jupyter kernel")
    parser.add_argument("--user", action="store_true", help="install to the current user")
    parser.add_argument("--sys-prefix", action="store_true",
                        help="install into this Python environment")
    args = parser.parse_args(argv)
    if args.user and args.sys_prefix:
        parser.error("choose either --user or --sys-prefix")
    with tempfile.TemporaryDirectory(prefix="semauri-kernelspec-") as directory:
        root = Path(directory)
        (root / "kernel.json").write_text(
            json.dumps({
                "argv": [sys.executable, "-m", "semauri_kernel", "-f", "{connection_file}"],
                "display_name": "Semauri",
                "language": "semauri",
                "metadata": {"debugger": False},
            }, indent=2) + "\n", encoding="utf-8"
        )
        target = KernelSpecManager().install_kernel_spec(
            directory, kernel_name="semauri", user=args.user,
            prefix=sys.prefix if args.sys_prefix else (None if args.user else sys.prefix),
            replace=True,
        )
    print(f"Installed Semauri kernel in {target}")


if __name__ == "__main__":
    main()
