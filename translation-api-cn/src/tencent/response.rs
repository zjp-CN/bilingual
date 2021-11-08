use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Response<'r> {
    #[serde(borrow)]
    #[serde(rename = "Response")]
    pub res: ResponseInner<'r>,
}

impl<'r> Response<'r> {
    /// 提取翻译内容。
    pub fn dst(&self) -> Result<&[&str], ResponseError> {
        match &self.res {
            ResponseInner::Ok { res, .. } => Ok(res),
            ResponseInner::Err { error, .. } => Err(error.clone()),
        }
    }

    /// 提取翻译内容。
    pub fn dst_owned(self) -> Result<Vec<String>, ResponseError> {
        match self.res {
            ResponseInner::Ok { res, .. } => Ok(res.into_iter().map(|x| x.into()).collect()),
            ResponseInner::Err { error, .. } => Err(error),
        }
    }

    /// 翻译内容是否为 `str` 类型。无翻译内容或出错时，返回 `None`。
    pub fn is_borrowed(&self) -> Option<bool> {
        match &self.res {
            ResponseInner::Ok { res, .. } if res.len() != 0 => Some(true),
            _ => None,
        }
    }
}

/// 响应的信息。要么返回翻译结果，要么返回错误信息。
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ResponseInner<'r> {
    Ok {
        #[serde(rename = "RequestId")]
        id:   &'r str,
        #[serde(rename = "Source")]
        from: &'r str,
        #[serde(rename = "Target")]
        to:   &'r str,
        #[serde(borrow)]
        #[serde(rename = "TargetTextList")]
        res:  Vec<&'r str>,
    },
    Err {
        #[serde(rename = "RequestId")]
        id:    &'r str,
        #[serde(rename = "Error")]
        error: ResponseError,
    },
}

/// 错误处理 / 错误码
///
/// see:
/// - https://cloud.tencent.com/document/product/551/30637
/// - https://cloud.tencent.com/api/error-center?group=PLATFORM&page=1
/// - https://cloud.tencent.com/document/product/551/40566
#[derive(Debug, Clone, Deserialize)]
pub struct ResponseError {
    #[serde(rename = "Code")]
    pub code: String,
    #[serde(rename = "Message")]
    pub msg:  String,
}

impl std::error::Error for ResponseError {}
impl std::fmt::Display for ResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
               "错误码：`{}`\n错误信息：`{}`\n错误含义：{}\n以上内容由腾讯云 API 返回",
               self.code,
               self.msg,
               self.solution())
    }
}

impl ResponseError {
    /// 参考：[错误码列表](https://cloud.tencent.com/document/product/551/30637)
    pub fn solution(&self) -> &str {
        match self.code.as_bytes() {
            b"ActionOffline" => "接口已下线。",
            b"AuthFailure.InvalidAuthorization" => "请求头部的 Authorization 不符合腾讯云标准。",
            b"AuthFailure.InvalidSecretId" => "密钥非法（不是云 API 密钥类型）。",
            b"AuthFailure.MFAFailure" => {
                "[MFA](https://cloud.tencent.com/document/product/378/12036) 错误。"
            }
            b"AuthFailure.SecretIdNotFound" => {
                "密钥不存在。请在控制台检查密钥是否已被删除或者禁用，如状态正常，\
                 请检查密钥是否填写正确，注意前后不得有空格。"
            }
            b"AuthFailure.SignatureExpire" => {
                "签名过期。Timestamp \
                 和服务器时间相差不得超过五分钟，请检查本地时间是否和标准时间同步。"
            }
            b"AuthFailure.SignatureFailure" => {
                "签名错误。签名计算错误，请对照调用方式中的签名方法文档检查签名计算过程。"
            }
            b"AuthFailure.TokenFailure" => "token 错误。",
            b"AuthFailure.UnauthorizedOperation" => "请求未授权。请参考",
            b"DryRunOperation" => "DryRun 操作，代表请求将会是成功的，只是多传了 DryRun 参数。",
            b"FailedOperation" => "操作失败。",
            b"InternalError" => "内部错误。",
            b"InvalidAction" => "接口不存在。",
            b"InvalidParameter" => "参数错误（包括参数格式、类型等错误）。",
            b"InvalidParameterValue" => "参数取值错误。",
            b"InvalidRequest" => "请求 body 的 multipart 格式错误。",
            b"IpInBlacklist" => "IP地址在黑名单中。",
            b"IpNotInWhitelist" => "IP地址不在白名单中。",
            b"LimitExceeded" => "超过配额限制。",
            b"MissingParameter" => "缺少参数。",
            b"NoSuchProduct" => "产品不存在",
            b"NoSuchVersion" => "接口版本不存在。",
            b"RequestLimitExceeded" => "请求的次数超过了频率限制。",
            b"RequestLimitExceeded.GlobalRegionUinLimitExceeded" => "主账号超过频率限制。",
            b"RequestLimitExceeded.IPLimitExceeded" => "IP限频。",
            b"RequestLimitExceeded.UinLimitExceeded" => "主账号限频。",
            b"RequestSizeLimitExceeded" => "请求包超过限制大小。",
            b"ResourceInUse" => "资源被占用。",
            b"ResourceInsufficient" => "资源不足。",
            b"ResourceNotFound" => "资源不存在。",
            b"ResourceUnavailable" => "资源不可用。",
            b"ResponseSizeLimitExceeded" => "返回包超过限制大小。",
            b"ServiceUnavailable" => "当前服务暂时不可用。",
            b"UnauthorizedOperation" => "未授权操作。",
            b"UnknownParameter" => "未知参数错误，用户多传未定义的参数会导致错误。",
            b"UnsupportedOperation" => "操作不支持。",
            b"UnsupportedProtocol" => "http(s) 请求协议错误，只支持 GET 和 POST 请求。",
            b"UnsupportedRegion" => "接口不支持所传地域。",
            b"FailedOperation.NoFreeAmount" => {
                "本月免费额度已用完，如需继续使用您可以在机器翻译控制台升级为付费使用。"
            }
            b"FailedOperation.ServiceIsolate" => "账号因为欠费停止服务，请在腾讯云账户充值。",
            b"FailedOperation.UserNotRegistered" => {
                "服务未开通，请在腾讯云官网机器翻译控制台开通服务。"
            }
            b"InternalError.BackendTimeout" => "后台服务超时，请稍后重试。",
            b"InternalError.ErrorUnknown" => "未知错误。",
            b"InternalError.RequestFailed" => "请求失败。",
            b"InvalidParameter.DuplicatedSessionIdAndSeq" => "重复的SessionUuid和Seq组合。",
            b"InvalidParameter.MissingParameter" => "参数错误。",
            b"InvalidParameter.SeqIntervalTooLarge" => "Seq之间的间隙请不要大于2000。",
            b"LimitExceeded.LimitedAccessFrequency" => "超出请求频率。",
            b"UnauthorizedOperation.ActionNotFound" => "请填写正确的Action字段名称。",
            b"UnsupportedOperation.AudioDurationExceed" => {
                "音频分片长度超过限制，请保证分片长度小于8s。"
            }
            b"UnsupportedOperation.TextTooLong" => {
                "单次请求text超过长度限制，请保证单次请求长度低于2000。"
            }
            b"UnsupportedOperation.UnSupportedTargetLanguage" => {
                "不支持的目标语言，请参照语言列表。"
            }
            b"UnsupportedOperation.UnsupportedLanguage" => "不支持的语言，请参照语言列表。",
            b"UnsupportedOperation.UnsupportedSourceLanguage" => "不支持的源语言，请参照语言列表。",
            _ => "未知错误。",
        }
    }
}

#[test]
fn response_test() {
    let success = r#"{"Response":{"RequestId":"7895050c-b0bd-45f2-ba88-c95c509020f2","Source":"en","Target":"zh","TargetTextList":["嗨","那里"]}}"#;
    assert_eq!(format!("{:?}", serde_json::from_str::<Response>(success).unwrap()),
               "Response { res: Ok(Success { id: \"7895050c-b0bd-45f2-ba88-c95c509020f2\", from: \
                \"en\", to: \"zh\", res: [\"嗨\", \"那里\"] }) }");
    let error = r#"{"Response":{"Error":{"Code":"AuthFailure.SignatureFailure","Message":"The provided credentials could not be validated. Please check your signature is correct."},"RequestId":"47546ee3-767c-4671-8f90-2c02c7484a42"}}"#;
    #[rustfmt::skip]
    assert_eq!(
               format!("{:?}", serde_json::from_str::<Response>(error).unwrap()),
               "Response { res: Err(ResponseErr { id: \"47546ee3-767c-4671-8f90-2c02c7484a42\", \
                error: ResponseError { code: \"AuthFailure.SignatureFailure\", \
                msg: \"The provided credentials could not be validated. \
                Please check your signature is correct.\" } }) }"
    );
}
