import os

members = [
    "nl/", "ir/", "ir2triple/", "ir2wasm/", "ir2x86/", "ofile/", "syntax/", "wasm/", "x86/"
]

def format_file(path):
    with open(path, "r") as f:
        c = f.read()
    
    c = c.replace("\t", "    ")

    with open(path, "w") as f:
        f.write(c)

def format_dir(path):
    for entry in os.scandir(path):
        if entry.is_file():
            if entry.name.endswith(".rs"):
                format_file(entry.path)
        elif entry.is_dir():
            format_dir(entry.path)

for member in members:
    format_dir(member)