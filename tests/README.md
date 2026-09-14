# Jolteon Integration Tests

FATE-like tests for Jolteon (see https://ffmpeg.org/fate.html).

Core concept:
- Ideally, test jolteon as a black box that takes an input and produces an output. For example, `jolteon play <file>`, with an ALSA config that has the default sink writing to a file, we then read some stats from the output file and compared against expected values, using simple fixtures such as tone-440.wav
- Some other tests can access the inner workings of Jolteon and test specific components, like the SingleTrackPlayer or MainPlayer, but still never mock anything in the code itself, and never test implementation details. Input and output is still raw audio, and we test on it.

Jolteon uses threads, as does CPAL internally. To avoid possible issues, we must either run tests sequentially or in parallel but isolated from each other. Current winning candidate for this is podman containers.

An example ALSA configuration:

```
pcm.!default {
    type file
    slave.pcm {
        type null
    }
    file "/tmp/jolteon-test-123/output.wav"
    format "wav"
}
```

We can run obtain some data from audio files with `ffmpeg`:

```
ffmpeg -i fixtures/tone-440hz.wav -af "astats=metadata=1:reset=0,ametadata=print" -f null -
```

A very primitive and fragile approach:

```nushell
» ffmpeg -i fixtures/tone-440hz.wav -af "astats=metadata=1:reset=0,ametadata=print" -f null - out+err>| lines | where $in =~ Parsed_astats | each { split row "] " | last } | skip until { $in == "Overall" } | skip 1 | to text
DC offset: 0.000000
Min level: -4095.000000
Max level: 4095.000000
Min difference: 1.000000
Max difference: 237.000000
Mean difference: 150.135772
RMS difference: 166.754918
Peak level dB: -18.063656
RMS level dB: -21.073778
RMS peak dB: -21.058087
RMS through dB: -23.078021
Flat factor: 0.000000
Peak count: 1120.000000
Abs Peak count: 1120.000000
Noise floor dB: -18.063656
Noise floor count: 93601.000000
Entropy: 0.634505
Bit depth: 12/16/16/16
Number of samples: 96000

» let stats = (
  ffmpeg -i fixtures/tone-440hz.wav
    -af "astats=metadata=1:reset=0,ametadata=print"
    -f null - 
  out+err>|lines
  | where $in =~ Parsed_astats
  | each { split row "] " | last }
  | skip until { $in == "Overall" }
  | skip 1
  | each {
      let parts = ($in | split row ":" | each { str trim })
      { ($parts | get 0): ($parts | get 1) }
    }
  | reduce --fold {} { |it, acc| $acc | merge $it }
)

» $stats | table -t none
 DC offset           0.000000     
 Min level           -4095.000000 
 Max level           4095.000000  
 Min difference      1.000000     
 Max difference      237.000000   
 Mean difference     150.135772   
 RMS difference      166.754918   
 Peak level dB       -18.063656   
 RMS level dB        -21.073778   
 RMS peak dB         -21.058087   
 RMS through dB      -23.078021   
 Flat factor         0.000000     
 Peak count          1120.000000  
 Abs Peak count      1120.000000  
 Noise floor dB      -18.063656   
 Noise floor count   93601.000000 
 Entropy             0.634505     
 Bit depth           12/16/16/16  
 Number of samples   96000
 
 » $stats."Peak level dB"
-18.063656
```
