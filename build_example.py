import os
import sys

available_examples = [
    "basic",
    "template",
    "nonconststate"
]

def print_examples():
    print("Available examples:")
    for example in available_examples:
        print(f"    {example}")

def pwsh(string):
    os.system(f"powershell.exe {string}")


if len(sys.argv) != 2:
    print("Invalid arguments. example usage:")
    print("    python build_example.py {example_name}")
    print_examples()
    
    exit(1)

example = sys.argv[1]

if example not in available_examples:
    print(f"{example} is not a known example")
    print_examples()

pwsh(f"cargo build --package {example}-example")
pwsh(f"rm plugins/{example}_example.dll")
pwsh(f"Move-Item -Path target-win/debug/{example}_example.dll -Destination plugins/{example}_example.dll")
