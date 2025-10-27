$ErrorActionPreference = 'Stop';
$toolsDir   = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"
$url64      = 'https://github.com/robottwo/aicred/releases/download/v0.1.0/keyfinder-windows-x86_64.zip'
$checksum64 = 'REPLACE_WITH_ACTUAL_SHA'

$packageArgs = @{
  packageName   = $env:ChocolateyPackageName
  unzipLocation = $toolsDir
  url64bit      = $url64
  checksum64    = $checksum64
  checksumType64= 'sha256'
}

Install-ChocolateyZipPackage @packageArgs

Install-BinFile -Name "keyfinder" -Path "$toolsDir\keyfinder.exe"