# scotoma
probably the first ever open source windows 11 ransomware with post quantum encryption

# disclaimer
this tool is strictly for educational and security research purposes only and should not be used in any malicious or unethical way. you are fully responsible for how and what you chose to do with this. i have no responsibility for any damage, loss of data, or any unintended consequences that may result from using this tool. by using it you agree fully to the disclaimer

# faq
1. what algorithm is used in the file encryption? xchacha-poly1305
2. how is this post quantum? it generates a random key to encrypt the files and encrypt that key with X25519 + ML-KEM-1024
3. is this undetected? idk probably not, i dont expect it to be
4. do i need a web server to use this? no
5. does this work on other windows versions? probably, ive never tried tho
6. will there ever be a version for linux? maybe
7. why is it called "scotoma"? scotoma is a blind spot in your eyes that can either be temporary or permanent, just like how this can encrypt files and leave them like that either temporarily or permanently

# how to use
1. go into `/scotoma/panel` and build it
2. click on the link it outputs
3. put in the settings you want and click "build"
4. it will build the encryptor and decryptor
5. they will be built do `/scotoma/.builds/latest-...`
