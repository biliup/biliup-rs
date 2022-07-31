import stream_gears

if __name__ == '__main__':
    stream_gears.upload(
        ["examples/test.mp4"],
        "cookies.json",
        "title",
        171,
        "tag",
        1,
        "source",
        "desc",
        "dynamic",
        "",
        None,
        stream_gears.UploadLine.Bda2,
        3,
    )
