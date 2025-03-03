# backend-takehome

Install rust: https://www.rust-lang.org/learn/get-started

To run the project, after installing rust, run 

`$ cargo run`

In same directory as the readme. 

To verify the project is working, once it's running, you should be able to

`$ curl http://localhost:3000` and see `{"data":"hello world"}`


## New Features
Added the following endpoints:

```
 $ curl 0.0.0.0:3000/api/jamf/devices 


{"devices":[{"device_id":"13","name":"cw-zsn-mac-1","model":"VirtualMac2,1","os":"macOS","os_is_latest":false},{"device_id":"14","name":"demo’s Virtual Machine","model":"VirtualMac2,1","os":"macOS","os_is_latest":false},{"device_id":"15","name":"demo’s Virtual Machine","model":"VirtualMac2,1","os":"macOS","os_is_latest":false},{"device_id":"12","name":"peter’s MacBook Air","model":"MacBook Air (M1, 2020)","os":"macOS","os_is_latest":false}]}
```

and 

```
$ curl --header "Content-Type: application/json" \
  --request POST \
  --data '{"username":"xyz","password":"xyz", "url": "https://security.stuff.com"}' \
  http://localhost:3000/api/jamf/credentials


{"username":"xyz","password":"xyz","url":"https://security.stuff.com"}
```

Useful references:
Axum documentation: https://docs.rs/axum/latest/axum/all.html#
Serde documentation: https://serde.rs/
Env vars in rust (please do not commmit environment variables to github): https://doc.rust-lang.org/book/ch12-05-working-with-environment-variables.html

