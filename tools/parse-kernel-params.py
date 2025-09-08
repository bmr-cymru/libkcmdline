#!/usr/bin/python3
from dataclasses import dataclass, field
from argparse import ArgumentParser
from typing import Dict, List
from os.path import basename
from pathlib import Path
from os import makedirs
import string
import toml
import sys


@dataclass
class Param:
    name: str = "INVALID"
    flags: List[str] = field(default_factory=list)
    desc: str = ""
    fmt: str = ""
    values: List[str] = field(default_factory=list)


def parse_flags(flags: str) -> List[str]:
    flags = flags.lstrip("[")
    flags = flags.strip("]")
    return flags.split(",")


def parse_values(values: str) -> List[str]:
    # We;re not doing neested enum syntax for literally one parameter
    # cgroup_no_v1= can just be a special snowflake for this.
    if values.startswith("{ {"):
        values = values.replace("{ {", "{")

    sep = "|" if "|" in values else ","
    return list(map(lambda x: x.strip("\t \"'"), values.strip("{}").split(sep)))

def process_kernel_parameters(kernel_params: Path, outdir: Path):
    param_chars = string.ascii_letters + string.digits
    params = {}
    with open(kernel_params, "r", encoding="utf8") as fp:
        param = None
        format_line = None
        while line := fp.readline():
            if line == "\n" and param:
                param.desc += "\n"
            if line.startswith("\t") and line[1] in param_chars:
                # Complete last Param object.
                if param:
                    if not param.fmt:
                        param.fmt = "flag"
                    params[param.name] = param
                    param = None
                # New parameter: get name, maybe fmt, flags and desc
                parts = line.split(maxsplit=2)
                name = parts[0]
                name_parts = name.split("=", maxsplit=1)
                name = name_parts[0]
                # Ugggh..
                if name == "sdw_mclk_divider" and name_parts[1] == "[SDW]":
                    flags = parse_flags(name_parts[1])
                    fmt = "<int>"
                else:
                    flags = parse_flags(parts[1]) if len(parts) > 1 else None
                    fmt = name_parts[1] if len(name_parts) == 2 and name_parts[1] else ""
                desc = parts[2].strip() if len(parts) > 2 else ""
                param = Param(name=name, flags=flags, desc=desc, fmt=fmt)
            if param and line.startswith("\t\t\t"):
                line = line.removeprefix("\t\t\t").rstrip()\

                # Handle Format: lines with or without {...} enum values.
                if line.startswith("Format: ") or line.startswith("{"):
                    if line.startswith("Format: "):
                        line = line.removeprefix("Format: ")
                        if "{"  not in line and "}" not in line and format_line is None:
                            param.fmt = line
                            continue

                # Glue format lines between {..[|,]..} together
                if line.startswith("{") and( "|" in line or "," in line) and line.endswith("}"):
                    param.values = parse_values(line)
                    param.fmt = "enum"
                elif line.startswith("{") and ("|" in line or "," in line):
                    format_line = line
                elif format_line and ("|" in line or "," in line) and line.endswith("}"):
                    format_line += line
                    param.values = parse_values(format_line)
                    param.fmt = "enum"
                    format_line = None
                elif format_line and ("|" in line or "," in line):
                    format_line += line
                elif line.startswith("{") and line.endswith("}"):
                    param.fmt = line.strip("{}")
                else:
                    param.desc += "\n" + line
    return params


def dump_kernel_parameter(param: Param):
    print(f"Name: {param.name}")
    print(f"Format: {param.fmt}")
    print(f"Flags: {", ".join(param.flags) if param.flags else ""}")
    print(f"Values: {", ".join(param.values) if param.values else ""}")
    print(f"Desc: {param.desc}\n")

def dump_kernel_parameters(params: Dict[str, Param]):
    for param in params.values():
        dump_kernel_parameter(param)


def main() -> int:
    parser = ArgumentParser(
        basename(sys.argv[0]),
        description="Bootstrap kernel/ database"
    )
    parser.add_argument(
        "parameters",
        metavar="PARAMETERS",
        type=Path,
        help="Path to the kernel-parameters.txt file to process",
    )
    parser.add_argument(
        "outdir",
        metavar="OUTDIR",
        type=Path,
        help="Output directory",
        default="database/"
    )
    parser.add_argument(
        "--dump-parameters",
        "--dumpparameters",
        "--dump",
        action="store_true",
        help="Dump parameter definitions to stdout",
    )
    args = parser.parse_args()

    params = args.parameters.expanduser()
    outdir = args.outdir.expanduser()
    makedirs(outdir, exist_ok=True)

    assert params.exists()
    assert outdir.exists()

    params = process_kernel_parameters(params, outdir)
    if args.dump_parameters:
        dump_kernel_parameters(params)


if __name__ == "__main__":
    main()
