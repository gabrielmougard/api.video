# Copyright 2022 gab
# 
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
# 
#     http://www.apache.org/licenses/LICENSE-2.0
# 
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

import os.path

from quompressor import compress

INTPUT_FILE = "./examples/kitchen-2048x2048.png"
OUTPUT_FILE = "./examples/kitchen-2048x2048-python.png"

if __name__ == "__main__":
    assert OUTPUT_FILE == compress(INTPUT_FILE, OUTPUT_FILE, width_=2048)
    
    in_size = os.path.getsize(INTPUT_FILE)
    out_size = os.path.getsize(OUTPUT_FILE)
print(
    f"The input file {INTPUT_FILE} size is {in_size} bytes\n" +
    f"The output file {OUTPUT_FILE} size is {out_size} bytes\n" +
    f"The compression ratio is : {1.0 - out_size/in_size}%\n"
)