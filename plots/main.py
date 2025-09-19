# import matplotlib.pyplot as plt
import json

def main():
    with open("../results/chromosome.json") as f:
        file_contents = f.read()
        raw = json.loads(file_contents)
        print(raw)
        

if __name__ == "__main__":
    main()
