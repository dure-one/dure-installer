## Android

### Requirements

* rust 1.81
* android ndk, sdk, java, llvm, kotlin, and gradle will install with build.sh

### Building
```bash
export ANDROID_NDK_HOME="path/to/ndk"
export ANDROID_HOME="path/to/sdk"

rustup target add aarch64-linux-android
cargo install cargo-ndk

cargo ndk -t arm64-v8a -o app/src/main/jniLibs/ build --release
gradle build
gradle installDebug
```

### Logcat

```bash
# clar logcat
$ adb logcat -c
# full logcat
$ adb logcat -v time -s *:V > fullcat.log
# app specific logcat
$ adb logcat -s Dure > uadcat.log
```

### Android Mipmap icons generation
https://easyappicon.com/

### Google Play Keys (Automated)

**Quick Start**: Use the automated script to generate all keys at once:

Download app public key(encryption_public_key.pem) from : https://play.google.com/console/u/0/developers/app/protect-with-play
* on this instruction we don't share upload key. upload key and release key is per app.

```bash
./googleplaykey.sh /path/to/encryption_public_key.pem [password]
```

This script:
- Checks for existing keystores (prevents overwrite)
- Generates `release.keystore` (upload key)
- Generates `app-dure-installer-signing.keystore` (app signing key)
- Exports `upload_certificate.pem`
- Downloads pepk.jar
- Creates encrypted .zip for Play Console

### Android keystore for github workflow

Export to base64 for CI/CD:
```bash
base64 release.keystore > release.keystore.base64
```

Set GitHub secrets:
- `KEYSTORE_BASE64=<output from above>`
- `STORE_PASSWORD=<UPLOAD_PASSWORD>`
- `KEY_PASSWORD=<UPLOAD_PASSWORD>`
- `KEY_ALIAS=upload`
- `GOOGLEPLAY_TRACK=qa`

for GOOGLEPLAY_TRACK, qa is internal track, alpha, beta are testing tracks, production is production track.

Create Service Account for Google Play autoupload
[Create Service Account](https://github.com/r0adkll/upload-google-play/tree/master#configure-access-via-service-accoun<Find>t)

### Upload signing key on google play

**Download encryption public key from Play Console**
- Go to App Integrity > App Signing
- Download encryption_public_key.pem to ./mobile/app/

**Upload to Play Console**
- Go to App Integrity > App Signing
- Upload the generated .zip file

**Key Summary**:
- **Upload Key** (`release.keystore`): signs APK locally
- **App Signing Keys** (one per app): Unique, managed by Google Play

### Reset Upload Key on Google Play Console

When changing upload key (security update, key compromise, or new keystore):

**Upload to Play Console**
- Go to App Integrity > Change Signing Key
- Select "Reset Upload Key"
- Choose reason (e.g., "분실됨" / "Lost", "손상됨" / "Compromised", "비밀번호를 잊어버림" / "Forgot password")
- Upload `upload_certificate.pem`
- Submit and wait for approval

### Submit app to Google Playstore
1. register google play console
2. create unique upload key, app signing key for each app (see "Upload signing key" section above)
3. add service account and upload key to github repository secrets.
4. run github workflow

### App testing
* Closed Testing Service Provider
https://www.testerscommunity.com/app-details/LOZEF4vnbuI4DQdaRfs1
* Google Play Console Pulishing Overview
https://play.google.com/console/u/1/developers/8469971848379081167/app/4976079442462544107/publishing
```
- android test link : https://play.google.com/store/apps/details?id=app.dure.installer
- additional test invitation link : https://play.google.com/apps/internaltest/4700175684927727957
- join on the web link : https://play.google.com/apps/testing/app.dure.installer
- youtube instructions : https://www.youtube.com/shorts/OuPw-hi4-c4
- setup guide : https://dure.pages.dev/docs/setup
- other instructions : https://dure.pages.dev/docs/usage
```

### Publish to Snapstore
```
$ sudo apt update && sudo apt install -y snapd
$ sudo systemctl enable --now snapd.socket
$ sudo snap install snapd
$ sudo snap install snapcraft --classic
$ sudo snap refresh snapd snapcraft
```

1. register snap name in https://snapcraft.io/snaps
2. update app settings in https://snapcraft.io/dure/releases

docs : https://documentation.ubuntu.com/snapcraft/stable/how-to/publishing/publish-a-snap/

### Submit app to Fdroid
1. fork fdroid data repository in gitlab (https://gitlab.com/fdroid/fdroiddata)
2. clone forked repository (git clone https://gitlab.com/nikescar/fdroiddata)
3. make branch with new package name (git checkout -b app.dure.installer)
4. add fdoid metadata (../deploy/fdroid/app.dure.installer.yml)
5. commit and push it to gitlab (git commit -a -m 'initial commit' && git push origin)
6. make PR to check build pipeline. (make sure PR Contents uses fdroid templates)

docs : https://f-droid.org/en/docs/

### Publish to Flathub(Flatpak)
1. fork flathub data repository(https://github.com/flathub/flathub) in github. uncheck clone "only master branch". 
2. clone "new-pr" branch(git clone https://github.com/nikescar/flathub -b new-pr) from the fork. and move to app name(app.dure.installer) branch (git checkout -b app.dure.installer)
3. copy files from the directory (../deploy/flatpak/*) and edit properly.
4. commit and make PR against flathub/flathub(new-pr branch) repository.
5. pass the test.

docs : https://docs.flathub.org/docs/for-app-authors/submission

### Publish to Huawei
https://docs.nhncloud.com/en/Mobile%20Service/IAP/en/console-huawei-guide/

### Publish to Msstore
https://partner.microsoft.com/en-us/dashboard/apps-and-games/overview

### Publish to Choco
https://mtirion.medium.com/publish-a-net-console-app-to-chocolatey-using-github-actions-29eaa60a8668

### Publish to Winget
https://github.com/isaacrlevin/winget-publish-action

### Publish with cloudsmith
https://cloudsmith.io/~nikescar/repos/dure/upload/

### Publish with Goreleaser

Homebrew : https://dev.to/hadlow/how-to-release-to-homebrew-with-goreleaser-github-actions-and-semantic-release-2gbb

