#!/usr/bin/python3
from tomlkit import comment, document, nl, string as tkstring, table
from dataclasses import dataclass, field
from argparse import ArgumentParser
from os import makedirs, fdatasync
from typing import Dict, List
from os.path import basename
from pathlib import Path
import logging
import string
import sys

log = logging.getLogger("__name__")

def _log_debug_harder(*args, **kwargs):
    if _DEBUG_HARDER:
        _log_debug(*args, **kwargs)


_log_debug = log.debug
_log_info = log.info
_log_warn = log.warning

formatter = logging.Formatter("%(levelname)s - %(message)s")
console_handler = logging.StreamHandler()
console_handler.setLevel(logging.INFO)
console_handler.setFormatter(formatter)
log.addHandler(console_handler)

_DEBUG_HARDER = False

_DB_TOP_DIR = Path("parameters/kernel")


@dataclass
class Param:
    name: str = "INVALID"
    flags: List[str] = field(default_factory=list)
    desc: str = ""
    fmt: str = ""
    values: List[str] = field(default_factory=list)
    subparams: Dict[str, "Param"] = field(default_factory=dict)


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


def process_kernel_parameters(kernel_params: Path):
    _log_info("Proccessing path: %s", kernel_params)
    param_chars = string.ascii_letters + string.digits
    params = {}
    line_count = 0
    with open(kernel_params, "r", encoding="utf8") as fp:
        param = None
        subparam = None
        format_line = None
        while line := fp.readline():
            line_count += 1

            _log_debug_harder(
                "processing line %d: %s, %s, %s",
                line_count,
                param,
                subparam,
                format_line
            )

            if line == "\n" and (param or subparam):
                if subparam:
                    subparam.desc += "\n"
                else:
                    param.desc += "\n"

            if line.startswith("\t") and line[1] in param_chars:
                # Complete any open sub-param before completing parma
                if subparam:
                    if subparam.desc.startswith("\n"):
                        subparam.desc = subparam.desc.lstrip("\n")
                    _log_debug(
                        "[%04d] Completing SUB-parameter %s (desc=%s, flags=%s, fmt=%s, values=%s)",
                        line_count,
                        subparam.name,
                        subparam.desc,
                        subparam.flags,
                        subparam.fmt,
                        subparam.values,
                    )
                    param.subparams[subparam.name] = subparam
                    subparam = None
                # Complete last Param object.
                if param:
                    if not param.fmt and not param.subparams:
                        param.fmt = "flag"
                    elif not param.fmt and param.subparams:
                        param.fmt = "complex"
                    if param.desc.startswith("\n"):
                        param.desc = param.desc.lstrip("\n")
                    _log_debug(
                        "[%04d] Completing PARAMETER %s (flags=%s, fmt=%s, values=%s)",
                        line_count,
                        param.name,
                        param.flags,
                        param.fmt,
                        param.values,
                    )
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
                    fmt = (
                        name_parts[1] if len(name_parts) == 2 and name_parts[1] else ""
                    )
                desc = parts[2].strip() if len(parts) > 2 else ""
                _log_debug(
                    "[%04d] New PARAMETER: %s, flags=%s, desc=%s, fmt=%s",
                    line_count,
                    name if f"{name}=" not in line else f"{name}=",
                    flags,
                    desc,
                    fmt,
                )
                param = Param(name=name, flags=flags, desc=desc, fmt=fmt)

            if param and line.startswith("\t\t") and line[2] in param_chars:
                line = line.removeprefix("\t\t\t").rstrip()
                if subparam:
                    if subparam.desc.startswith("\n"):
                        subparam.desc = subparam.desc.lstrip("\n")
                    _log_debug(
                        "[%04d] Completing SUB-parameter %s (desc=%s, flags=%s, fmt=%s, values=%s)",
                        line_count,
                        subparam.name,
                        subparam.desc,
                        subparam.flags,
                        subparam.fmt,
                        subparam.values,
                    )
                    param.subparams[subparam.name] = subparam
                    subparam = None
                # New parameter: get name, maybe fmt, flags and desc
                parts = line.split(maxsplit=2)
                name = parts[0]
                name_parts = name.split("=", maxsplit=2)
                name = name_parts[0]
                flags = (
                    parse_flags(parts[1])
                    if (len(parts) > 1 and parts[1].startswith("["))
                    else None
                )
                fmt = (
                    name_parts[1]
                    if (len(name_parts) == 2 and name_parts[1] and "=" in line)
                    else ""
                )
                desc = " ".join(parts[1:]).strip() if len(parts) > 2 else ""
                _log_debug(
                    "[%04d] New SUB-parameter: %s, flags=%s, desc=%s, fmt=%s",
                    line_count,
                    name,
                    flags,
                    desc,
                    fmt,
                )
                subparam = Param(name=name, flags=flags, desc=desc, fmt=fmt)

            if param and subparam and line.startswith("\t\t\t\t"):
                line = line.removeprefix("\t\t\t\t").rstrip()
                _log_debug(
                    "[%04d] Continuing description for SUBparam %s: %s",
                    line_count,
                    subparam.name,
                    line,
                )
                subparam.desc += "\n" + line

            if param and subparam and line.startswith("\t\t\t"):
                line = line.removeprefix("\t\t\t").rstrip()
                _log_debug(
                    "[%04d] Continuing description for SUBparam %s: %s",
                    line_count,
                    subparam.name,
                    line,
                )
                subparam.desc += "\n" + line

            if param and line.startswith("\t\t\t"):
                line = line.removeprefix("\t\t\t").rstrip()

                # Handle Format: lines with or without {...} enum values.
                if line.startswith("Format: ") or line.startswith("{"):
                    if line.startswith("Format: "):
                        line = line.removeprefix("Format: ")
                        if "{" not in line and "}" not in line and format_line is None:
                            _log_debug(
                                "[%04d] Found in-line format descroption: %s",
                                line_count,
                                line,
                            )
                            param.fmt = line
                            continue

                # Glue format lines between {..[|,]..} together
                if (
                    line.startswith("{")
                    and ("|" in line or "," in line)
                    and line.endswith("}")
                ):
                    param.values = parse_values(line)
                    param.fmt = "enum"
                    _log_debug(
                        "[%04d] Handling one-line enum block {...} %s",
                        line_count,
                        param.values,
                    )
                elif line.startswith("{") and ("|" in line or "," in line):
                    _log_debug(
                        "[%04d] Entering Format enum block {...: %s", line_count, line
                    )
                    format_line = line
                elif (
                    format_line and ("|" in line or "," in line) and line.endswith("}")
                ):
                    format_line += line
                    param.values = parse_values(format_line)
                    param.fmt = "enum"
                    _log_debug(
                        "[%04d] Exiting Format enum block ...} %s",
                        line_count,
                        param.values,
                    )
                    format_line = None
                elif format_line and ("|" in line or "," in line):
                    _log_debug(
                        "[%04d] Continuing Format enum block ..[|,].. %s",
                        line_count,
                        line,
                    )
                    format_line += line
                elif line.startswith("{") and line.endswith("}"):
                    _log_debug(
                        "[%04d] Handling non-enum format line: %s", line_count, line
                    )
                    param.fmt = line.strip("{}")
                else:
                    _log_debug(
                        "[%04d] Continuing description for param %s: %s",
                        line_count,
                        param.name,
                        line,
                    )
                    param.desc += "\n" + line.lstrip()
    return params

def write_kernel_parameter(param: Param, db_dir: Path):
    param_dir = db_dir / param.name
    makedirs(param_dir, exist_ok=True)
    param_file = param_dir / "definition.toml"

    if param_file.exists():
        param_file.unlink()

    doc = document()
    doc.add(comment("This is a libKCmdline definition document."))
    doc.add(nl())
    doc.add("title", f"{param.name} - definition.toml")
    doc.add("name", param.name)
    doc.add("processor", "kernel")
    doc.add("description", tkstring(param.desc, literal=True, multiline=True))
    syntax = table()
    syntax.add("type", param.fmt)
    syntax.add("format", param.fmt)  # FIXME
    syntax.add("choices", param.values)
    syntax.add("allow_empty", True)  # FIXME
    doc.add("syntax", syntax)

    with open(param_dir / "definition.toml", "w", encoding="utf8") as fd:
        fd.write(doc.as_string())
        fd.flush()
        fdatasync(fd.fileno())
    for (subname, subparam) in param.subparams.items():
        _log_debug("Writing subparameter '%s'", subname)
        write_kernel_parameter(subparam, param_dir)


def write_kernel_parameters(params: List[Param], db_dir: Path):
    for (name, param) in params.items():
        _log_debug("Writing parameter '%s'", name)
        write_kernel_parameter(param, db_dir)


def dump_kernel_parameter(param: Param):
    print(f"  Name: {param.name}")
    print(f"Format: {param.fmt}")
    print(f" Flags: {", ".join(param.flags) if param.flags else ""}")
    print(f"Values: {", ".join(param.values) if param.values else ""}")
    print(f"  Desc: {param.desc}\n")
    if param.subparams:
        print(f" Subparams:")
        for subparam in param.subparams.values():
            print(f"     Name: {param.name}")
            print(f"  SubName: {subparam.name}")
            print(f"   Format: {subparam.fmt}")
            print(f"    Flags: {", ".join(subparam.flags) if subparam.flags else ""}")
            print(f"   Values: {", ".join(subparam.values) if subparam.values else ""}")
            print(f"     Desc: {subparam.desc}\n")


def dump_kernel_parameters(params: Dict[str, Param]):
    for param in params.values():
        dump_kernel_parameter(param)


def main() -> int:
    parser = ArgumentParser(
        basename(sys.argv[0]), description="Bootstrap kernel/ database"
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
        default="database/",
    )
    parser.add_argument(
        "--dump-parameters",
        "--dumpparameters",
        "--dump",
        action="store_true",
        help="Dump parameter definitions to stdout",
    )
    parser.add_argument(
        "--verbose",
        "-v",
        action="store_true",
        help="Enable debug logging",
    )
    parser.add_argument(
        "--debug-harder",
        "-d",
        action="store_true",
        help="Enable debug logging",
    )
    args = parser.parse_args()

    params = args.parameters.expanduser()
    outdir = args.outdir.expanduser()
    db_dir = outdir / _DB_TOP_DIR
    makedirs(db_dir, exist_ok=True)

    assert params.exists()
    assert db_dir.exists()

    if args.debug_harder:
        global _DEBUG_HARDER
        _DEBUG_HARDER = True

    if args.verbose:
        log.setLevel(logging.DEBUG)
        console_handler.setLevel(logging.DEBUG)

    params = process_kernel_parameters(params)
    if args.dump_parameters:
        dump_kernel_parameters(params)

    write_kernel_parameters(params, db_dir)

    print(params["pci"].name)
    print(params["pci"].fmt)
    print(f"**{params["pci"].desc}**")
    print(params["pci"].values)

if __name__ == "__main__":
    main()
