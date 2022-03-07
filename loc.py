import os

def process_file(path):
    if not path.endswith(".rs"): return 0
    
    count = 0
    with open(path, "r") as f:
        while True:
            line = f.readline()
            if line is EOF: break

            if len(line.strip()) > 0:
                count += 1
    
    print(f"{count}\t{path}")
    return count

def scan(path):
    print(path)
    count = 0
    for ent in os.scandir(path):
        if ent.is_dir():
            count += scan(ent.path)
        else:
            count += process_file(ent.path)
    return count

scan(".")