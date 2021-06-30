# untilfail
Run a command over and over until it fails, and collect meta-data.
It is a way to check reliability of a system or a program by running something over and over until it fails.

I have written scripts to do this very thing many times, let's see if this program can replace doing it again.


## usage
```
USAGE:
    untilfail [FLAGS] [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help          Prints help information
    -k, --keep-going    Keep going when command fails
    -V, --version       Prints version information

OPTIONS:
    -d, --delay <delay>     [default: 1]
    -l, --log <log>
```

## example

```
$ untilfail  -k  -l mylog -- pytest --verbose -k test_radio
============================= test session starts ==============================
platform linux -- Python 3.9.5, pytest-6.2.2, py-1.10.0, pluggy-0.13.1 -- /usr/bin/python3
cachedir: .pytest_cache
rootdir: /home/jonasdn/sandbox/crazyflie-testing, configfile: pytest.ini
collecting ... collected 16 items / 11 deselected / 5 selected

tests/QA/test_radio.py::TestRadio::test_latency_small_packets[default]
-------------------------------- live log call ---------------------------------
INFO     test_radio:test_radio.py:94 latency: 2.9213428497314453
PASSED                                                                   [ 20%]
tests/QA/test_radio.py::TestRadio::test_latency_big_packets[default]
-------------------------------- live log call ---------------------------------
INFO     test_radio:test_radio.py:94 latency: 4.689216613769531
PASSED                                                                   [ 40%]
tests/QA/test_radio.py::TestRadio::test_bandwidth_small_packets[default]
-------------------------------- live log call ---------------------------------
INFO     test_radio:test_radio.py:130 bandwidth: 1068.1911949121723
PASSED                                                                   [ 60%]
tests/QA/test_radio.py::TestRadio::test_bandwidth_big_packets[default]
-------------------------------- live log call ---------------------------------
INFO     test_radio:test_radio.py:130 bandwidth: 470.7946626485522
PASSED                                                                   [ 80%]
tests/QA/test_radio.py::TestRadio::test_reliability[default]
-------------------------------- live log call ---------------------------------
INFO     test_radio:test_radio.py:130 bandwidth: 1072.7992345566986
PASSED                                                                   [100%]

====================== 5 passed, 11 deselected in 35.10s =======================
☆✹꙳* Iterations: 1, failures: 0 ✰🟈✵☆
============================= test session starts ==============================
```
