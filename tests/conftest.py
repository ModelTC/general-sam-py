def parse_from_readme():
    """
    Adapted from https://github.com/boxed/pytest-readme

    Copyright (c) 2020, Anders Hovmöller

    Under the BSD 3-Clause "New" or "Revised" License.
    """
    with open(
        "tests/test_readme.py",
        "w",
        encoding="utf8",
    ) as out, open("README.md", encoding="utf8") as readme:
        output, mode = [], None

        for i, line in enumerate(readme.readlines()):
            output.append("\n")

            if mode is None and line.strip() == "```python":
                mode = "first_line"
                output[i] = "def test_line_%s():\n" % i
                continue

            if line.strip() == "```":
                if mode == "doctest":
                    output[i] = '    """\n'
                mode = None
                continue

            if mode == "first_line":
                if line.strip() == "":
                    mode = None
                    output[i - 1] = "\n"
                    continue

                if line.strip().startswith(">>>"):
                    mode = "doctest"
                    output[i - 2] = (
                        output[i - 1][:-1] + "  " + output[i - 2]
                    )  # move the def line one line up
                    output[i - 1] = '    """\n'
                else:
                    mode = "test"

            if mode in ("doctest", "test"):
                output[i] = "    " + line
            else:
                output[i] = "# %s" % line

        out.writelines(output)


parse_from_readme()
