import stream_gears

if __name__ == '__main__':
    stream_gears.upload(
        ["examples/test.mp4",
         "examples/test2.mp4"],
        "cookies.json",
        "52532525",
        171,
        "演示",
        1,
        "",
        "",
        "",
        "",
        0,
        0,
        0,
        0,
        True,
        False,
        False,
        3,
        [],
        None,
        stream_gears.UploadLine.Qn,
    )
