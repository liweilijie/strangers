use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct MedicinalBackendQueryArg {
    pub page: Option<u32>,
    pub keyword: Option<String>,
    pub msg: Option<String>,
    pub is_del: Option<bool>,
    pub expired: Option<u8>,
    // pub order: Option<OrderKey>,
}

impl MedicinalBackendQueryArg {
    pub fn page(&self) -> u32 {
        match &self.page {
            Some(p) => *p,
            None => 0,
        }
    }
    pub fn keyword(&self) -> &str {
        match &self.keyword {
            Some(s) => s,
            None => "",
        }
    }
    pub fn is_del(&self) -> bool {
        match &self.is_del {
            Some(b) => *b,
            None => false,
        }
    }
    // 0 表示没有输入参数, 不需要处理,只有大于0的时候才需要处理.
    // 1 表示已经过期的数据
    // 2 表示1个月过期的
    // 3 表示2个月过期的
    // 4 表示3个月过期的
    // 5 表示4个月过期的
    // 6 表示5个月过期的
    pub fn expired(&self) -> u8 {
        match &self.expired {
            Some(b) => *b,
            None => 0,
        }
    }

    // 排序方式
    // 默认以有效期排序
    // 可以以类目排序
    // 刚 upload之后以 create_at 排序
    // pub fn order(&self) -> String {
    //     match &self.order {
    //         Some(s) => s.to_string(),
    //         None => "validity".to_string(),
    //     }
    // }
}

#[derive(Deserialize, Debug)]
pub enum OrderKey {
    Validity,
    Category,
    CreatedAt,
}

impl OrderKey {
    pub fn to_string(&self) -> String {
        match self {
            OrderKey::Validity => "validity".to_string(),
            OrderKey::Category => "category".to_string(),
            OrderKey::CreatedAt => "created_at".to_string(),
        }
    }
}

impl From<&str> for OrderKey {
    fn from(s: &str) -> Self {
        match s {
            "validity" => OrderKey::Validity,
            "category" => OrderKey::Category,
            "created_at" => OrderKey::CreatedAt,
            _ => OrderKey::Validity,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct BackendQueryArg {
    pub page: Option<u32>,
    pub keyword: Option<String>,
    pub msg: Option<String>,
    pub is_del: Option<bool>,
}

impl BackendQueryArg {
    pub fn page(&self) -> u32 {
        match &self.page {
            Some(p) => *p,
            None => 0,
        }
    }

    pub fn keyword(&self) -> &str {
        match &self.keyword {
            Some(s) => s,
            None => "",
        }
    }

    pub fn is_del(&self) -> bool {
        match &self.is_del {
            Some(b) => *b,
            None => false,
        }
    }
}
