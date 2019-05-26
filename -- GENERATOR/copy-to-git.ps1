#############
# Locations #
#############

$original = "C:\muzudho\projects_rust\rust-kifuwarabe-wcsc29-lib"
$deploy   = "C:\Users\�ނ��ł�\Documents\GitHub\rust-kifuwarabe-wcsc29-lib";

######
# Go #
######

# �t�@�C���̍폜�B
function Remove-File($dst) {
	if (Test-Path $dst) {
		Write-Host "Delete  : [$($dst)]."
		Remove-Item $dst
	}
}

# �f�B���N�g���[�̍폜�B
function Remove-Dir($dst) {
	if (Test-Path $dst) {
		Write-Host "Delete  : [$($dst)] directory."
		Remove-Item $dst -Recurse
	}
}

# �t�@�C���̃R�s�[�B
function Copy-File($src, $dst) {
	Write-Host "Copy    : [$($src)] --> [$($dst)]."
	Copy-Item $src $dst
}

# �f�B���N�g���[�̃R�s�[�B
function Copy-Dir($src, $dst) {
	Write-Host "Copy    : [$($src)] --> [$($dst)] directory."
	Copy-Item $src $dst -Recurse
}

##
 # Note: Not trailer slash.
 ##

Remove-Dir  "$($deploy)\-- GENERATOR"
Remove-Dir  "$($deploy)\.vscode"
Remove-Dir  "$($deploy)\docs"
Remove-Dir  "$($deploy)\examples"
Remove-Dir  "$($deploy)\src"

Remove-File "$($deploy)\#memo.txt"
Remove-File "$($deploy)\.gitignore"
Remove-File "$($deploy)\Cargo.lock"
Remove-File "$($deploy)\Cargo.toml"
Remove-File "$($deploy)\kifuwarabe-wcsc29-exe-config.json"
Remove-File "$($deploy)\LICENSE"
Remove-File "$($deploy)\README.md"

Copy-Dir    "$($original)\-- GENERATOR"                      "$($deploy)\-- GENERATOR"
Copy-Dir    "$($original)\.vscode"                           "$($deploy)\.vscode"
Copy-Dir    "$($original)\docs"                              "$($deploy)\docs"
Copy-Dir    "$($original)\examples"                          "$($deploy)\examples"
Copy-Dir    "$($original)\src"                               "$($deploy)\src"

Copy-File   "$($original)\#memo.txt"                         "$($deploy)\#memo.txt"
Copy-File   "$($original)\.gitignore"                        "$($deploy)\.gitignore"
Copy-File   "$($original)\Cargo.lock"                        "$($deploy)\Cargo.lock"
Copy-File   "$($original)\Cargo.toml"                        "$($deploy)\Cargo.toml"
Copy-File   "$($original)\kifuwarabe-wcsc29-exe-config.json" "$($deploy)\kifuwarabe-wcsc29-exe-config.json"
Copy-File   "$($original)\LICENSE"                           "$($deploy)\LICENSE"
Copy-File   "$($original)\README.md"                         "$($deploy)\README.md"

pause