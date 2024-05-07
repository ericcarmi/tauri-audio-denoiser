#!/bin/zsh
num_filters=-1;

input="./src-tauri/src/constants.rs"
while IFS= read -r line
do
  # echo "$line"
  if [[ $line == *"NUM_FILTERS"* ]]; then
    num_filters=$(echo $line | tail -c 3 | head -c 1);
    break;
  fi
done < "$input"

# should check if num_filters is not -1

string="CREATE TABLE FILTERBANK (
  id INTEGER PRIMARY KEY
  stereo_choice TEXT NOT NULL,
";

for i in $(seq 1 $num_filters);
do
  if [ $i -eq $num_filters ]; then
    comma=""
  else
    comma=","
  fi
  string+="  bpf_gain_$i REAL,\n" 
  string+="  bpf_freq_$i REAL,\n" 
  string+="  bpf_Q_$i REAL$comma\n" 
done

string+=");"

echo $string 
