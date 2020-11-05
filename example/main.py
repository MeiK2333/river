from pathlib import Path

import grpc

import river_pb2
import river_pb2_grpc


def judge(path, language):
    if language == river_pb2.C:
        filename = "main.c"
    elif language == river_pb2.Cpp:
        filename = "main.cpp"
    elif language == river_pb2.Python:
        filename = "main.py"
    elif language == river_pb2.Java:
        filename = "Main.java"
    elif language == river_pb2.Rust:
        filename = "main.rs"
    elif language == river_pb2.Go:
        filename = "main.go"
    with open(path.joinpath(filename), "rb") as fr:
        code = fr.read()
    with open(path.joinpath("in.txt"), "rb") as fr:
        in_data = fr.read()
    with open(path.joinpath("out.txt"), "rb") as fr:
        out_data = fr.read()
    # compile
    yield river_pb2.JudgeRequest(
        language=language,
        judge_type=river_pb2.Standard,
        compile_data=river_pb2.CompileData(code=code),
    )
    # judge
    yield river_pb2.JudgeRequest(
        language=language,
        judge_type=river_pb2.Standard,
        judge_data=river_pb2.JudgeData(
            in_data=in_data, out_data=out_data, time_limit=10000, memory_limit=65535
        ),
    )


def run():
    with grpc.insecure_channel("localhost:4003") as channel:
        stub = river_pb2_grpc.RiverStub(channel)
        for path in Path("java").iterdir():
            print(f"开始评测 {path}")
            for item in stub.Judge(judge(path, river_pb2.Java)):
                print(item)
            print(f"{path} 评测完成")
        for path in Path("c").iterdir():
            print(f"开始评测 {path}")
            for item in stub.Judge(judge(path, river_pb2.C)):
                print(item)
            print(f"{path} 评测完成")
        for path in Path("cpp").iterdir():
            print(f"开始评测 {path}")
            for item in stub.Judge(judge(path, river_pb2.Cpp)):
                print(item)
            print(f"{path} 评测完成")
        for path in Path("py").iterdir():
            print(f"开始评测 {path}")
            for item in stub.Judge(judge(path, river_pb2.Python)):
                print(item)
            print(f"{path} 评测完成")
        for path in Path("rust").iterdir():
            print(f"开始评测 {path}")
            for item in stub.Judge(judge(path, river_pb2.Rust)):
                print(item)
            print(f"{path} 评测完成")
        for path in Path("go").iterdir():
            print(f"开始评测 {path}")
            for item in stub.Judge(judge(path, river_pb2.Go)):
                print(item)
            print(f"{path} 评测完成")


if __name__ == "__main__":
    run()
