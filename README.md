# 陌生人

物资入库出库清单。利用 tokio,axum 来实现。

- 后台登录页面。
- 登录鉴权功能。
- 展示所有物资。
- 利用上传文件的方式添加物资入库。
- 手动添加物资入库。
- 删除过期物资。

## error的处理
这里面的错误处理让我学到很多,没有用到`thiserror`这个库,但是却很好的定义了各种错误类型,很是方便是自由.

## config库
`config`库也很方便,还有如何读取数组的值这个问题还没有想好,不过也可以作为字符串返回再自己解板,但是这种方式处理始终不安逸.
## postgres需要支持数据库的DateTime类型解码
[timestamp在postgres](https://sunnysab.cn/2020/03/01/Use-Of-Timestamp-With-Postgres-In-Rust/)在`postgres`目录下的`Cargo.toml`中找到了支持的`features`：

```toml
[features]
with-bit-vec-0_6 = ["tokio-postgres/with-bit-vec-0_6"]
with-chrono-0_4 = ["tokio-postgres/with-chrono-0_4"]
with-eui48-0_4 = ["tokio-postgres/with-eui48-0_4"]
with-geo-types-0_4 = ["tokio-postgres/with-geo-types-0_4"]
with-serde_json-1 = ["tokio-postgres/with-serde_json-1"]
with-uuid-0_8 = ["tokio-postgres/with-uuid-0_8"]
[badges.circle-ci]
repository = "sfackler/rust-postgres"
```

所以在我们项目中需要增加对`chrono`时间解析的features:
```toml
tokio-postgres = {version = "0.7", features = ["with-chrono-0_4"] }
```

## postgres的pgadmin
http://222.213.23.231:5050/browser/#

## 之前的代码有一个bug,当cookie有多个的时候在HeadeersMap之中读不完全,所以引用了另外一个友好处理cookie的库.
```toml
tower-cookies = "0.5.1"
```


## 参考的项目

- [https://github.com/axumrs/todo.git](https://github.com/axumrs/todo.git)
- [https://github.com/axumrs/roaming-axum](https://github.com/axumrs/roaming-axum)
- [https://github.com/axumrs/axum-rs](https://github.com/axumrs/axum-rs)
- [axum官方的例子](https://github.com/tokio-rs/axum/tree/main/examples)
- [postgres事务相关的接口](https://axum.rs/topic/todo-service/log-and-refactor)
- [error处理](https://github.com/RustMagazine/rust_magazine_2021/blob/main/src/chapter_2/rust_error_handle_and_log.md)