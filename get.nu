#!/bin/env -S nu

def print-list []: list<string> -> nothing {
  each { "- " + $in } | str join "\n" | print
  print ""
}

def print-warning [msg: string]: nothing -> nothing {
  [
    (ansi rb)
    "Warning:"
    (ansi reset)
    " "
    $msg
    "\n"
  ] | str join | print
}

def print-error [msg: string]: nothing -> nothing {
  [
    (ansi bg_red)
    "Error:"
    (ansi reset)
    " "
    $msg
    "\n"
  ] | str join | print
}

def print-quote [msg: string]: nothing -> nothing {
  print (ansi grey)
  $msg | str trim -c "\n" | split row "\n" | each { "> " + $in } | str join "\n" | print
  print (ansi reset)
}

def ansi-code []: string -> string {
  [
    (ansi "#f000f0")
    "`"
    $in
    "`"
    (ansi reset)
  ] | str join
}

print $"🎶 (ansi default_bold)Downloading and installing Jolteon(ansi reset) 🎶"
print ""

let previous = which -a jolteon

if ($previous | is-not-empty) {
  print-warning "there already is a `jolteon` binary available at the following location(s):"
  $previous | get path | print-list
  print ""
  # exit
}

let url = 'https://api.github.com/repos/lautarodragan/jolteon/releases/latest'
let release = http get $url
# let release = gh api --method=GET "repos/lautarodragan/jolteon/releases/latest" | from json
# let release = open latest.json
let assets_urls = ($release | get assets.browser_download_url)

let kernel = uname | get kernel-name
let arch = uname | get machine

let artifact_url = ($assets_urls | find -n -i $kernel | find -n -i $arch)

# if (true or ($artifact_url | is-empty)) {
if ($artifact_url | is-empty) {
  print-error "could not find an artifact do download for your OS"
  print $"OS name: ($kernel)"
  print $"Arch: ($arch)"
  print "Available assets:"
  $assets_urls | print-list
  print "If you think this is an error, please report it:"
  print "https://github.com/lautarodragan/jolteon/issues/new"
  exit 1
} else if ($artifact_url | length) > 2 {
  print-warning "more than one URL matches the kernel + arch"
  print "The following artifacts will be ignored:"
  $artifact_url | skip 1 | print-list
}

let artifact_url = ($artifact_url | first)
let basename = $artifact_url | path basename
let folder = mktemp --tmpdir --directory --suffix .jolteon
let file = $folder | path join $basename

print $"Downloading ($basename)\n  from ($artifact_url)\n  to ($file)\n"
http get $artifact_url | save $file

let app = tar vxf $file -C $folder
let app = $folder | path join $app
print $"Extracted to ($app)"
print ""

chmod +x $app

print $"Running ('jolteon version' | ansi-code) as a basic test"

let output = run-external $app version | complete

if $output.exit_code > 0 {
  print-error "jolteon exited with a non-zero status code"
  print $"Status code was: ($output.exit_code)"

  if ($output.stdout | is-empty) {
    print "Standard output was empty."
  } else {
    print "Standard output was:"
    print-quote $output.stdout
    print ""
  }

  if ($output.stderr | is-empty) {
    print "Error output was empty."
  } else {
    print "Standard error was:"
    print-quote $output.stderr
    print ""
  }

  print "Aborting installation."
  exit 1
}

let stdout = $output.stdout | str trim | str trim -c "\n"

if ($stdout | str contains -i "jolteon") {
  print $"✅ Output was: ($stdout)"
} else {
  print $"❌ Output was: ($stdout)"
  print "Aborting installation."
  exit 1
}

let target_dir = $nu.home-path | path join ".local" "bin" # ~/.local/bin
let target_path = $target_dir | path join "jolteon" # ~/.local/bin/jolteon

print $"Moving jolteon to ($target_path)"

# Ensure ~/.local/bin exists. Safe because it's a no-op if it already exists, and Nu's `mkdir` handles sub-paths, like `mkdir -p`.
mkdir $target_dir

if ($target_path | path exists) {
  loop {
    print ""
    print $"Target path already exists. (ansi default_bold)(ansi default_underline)What would you like to do?(ansi reset)"
    let answer = [
      "Backup the existing file"
      "Replace the existing file"
      "Exit the installer without finishing the installation"
    ] | input list -i
    match $answer {
      0 => { # Backup
        let backup_dir = mktemp -d -p $target_dir
        print $"Moving current jolteon to ($backup_dir)"
        mv $target_path $backup_dir
        break
      }
      1 => { # Replace
        print $"This will overwrite the existing ($target_path) with the one we just downloaded."
        print "Are you sure?"
        if (["No. Let me choose another option.", "Yes. Overwrite it."] | input list -i) == 1 {
          print "Overwriting existing jolteon"
          break
        }
      }
      2 => { # Abort
        print "Installation cancelled."
        exit 2
      }
    }
  }
}

print $"Moving ($app) to ($target_path)"
mv $app $target_path

let which_jolteon = which -a jolteon

if ($which_jolteon | is-empty) {
  # TODO: warning / further checks
}

if $target_dir not-in $env.PATH {
  print $"It looks like ($target_dir) isn't in your $env.PATH"
  print $"Try adding it to your Nushell configuration:"
  print $"'$env.PATH ++= [($nu.home-path | path join .local bin)]' | save --append ($nu.config-path)"
}

print ""
print "🎶 Jolteon installed successfully 🎶"
print $"Try running ('jolteon' | ansi-code) in your terminal"