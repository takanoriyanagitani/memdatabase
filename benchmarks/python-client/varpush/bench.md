# Simple benchmarks

## Pushing a value

| loops  | elapsed(apple m2) | elapsed(intel) |
|:------:|:-----------------:|:--------------:|
| 1      |  0.1 ms           |   0.1 ms       |
| 16     |  2.0 ms           |   1.1 ms       |
| 128    | 11   ms           |   9   ms       |
| 1 Ki   | 88   ms           |  63   ms       |
| 16 Ki  |  1.4 s            | 912   ms       |
| 128 Ki | 10.7 s            | 7.3   s        |

## Pushing values

| # values per push | loops | elapsed(intel) | rps | values / s |
|:-----------------:|:-----:|:--------------:|:---:|:----------:|
| 1                 | 5,000 | 302 ms         | 17K |     17 K   |
| 16                | 5,000 | 286 ms         | 17K |    280 K   |
| 128               | 5,000 | 308 ms         | 16K |  2,078 K   |
| 1,024             | 5,000 | 476 ms         | 11K | 10,756 K   |
| 16,384            |   500 | 531 ms         |  1K | 15,427 K   |
| 131,072           |    50 | 311 ms         | 161 | 21,073 K   |
