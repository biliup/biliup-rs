from typing import Dict, List, Optional
from enum import Enum

from .pyobject import Segment, Credit


def download(url: str,
             header_map: Dict[str, str],
             file_name: str,
             segment: Segment) -> None:
    """
    下载视频

    :param str url: 视频地址
    :param Dict[str, str] header_map: HTTP请求头
    :param str file_name: 文件名格式
    :param Segment segment: 视频分段设置
    """


def login_by_cookies() -> bool:
    """
    cookie登录

    :return: 是否登录成功
    """


def send_sms(country_code: int, phone: int) -> str:
    """
    发送短信验证码

    :param int country_code: 国家/地区代码
    :param int phone: 手机号
    :return: 短信登录JSON信息
    """


def login_by_sms(code: int, ret: str) -> bool:
    """
    短信登录

    :param int code: 验证码
    :param str ret: 短信登录JSON信息
    :return: 是否登录成功
    """


def get_qrcode() -> str:
    """
    获取二维码

    :return: 二维码登录JSON信息
    """


def login_by_qrcode(ret: str) -> bool:
    """
    二维码登录

    :param str ret: 二维码登录JSON信息
    :return: 是否登录成功
    """


def login_by_web_cookies(sess_data: str, bili_jct: str) -> bool:
    """
    网页Cookie登录1

    :param str sess_data: SESSDATA
    :param str bili_jct: bili_jct
    :return: 是否登录成功
    """


def login_by_web_qrcode(sess_data: str, dede_user_id: str) -> bool:
    """
    网页Cookie登录2

    :param str sess_data: SESSDATA
    :param str dede_user_id: DedeUserID
    :return: 是否登录成功
    """


class UploadLine(Enum):
    """上传线路"""

    Bda2 = 1
    """百度upos"""

    Ws = 2
    """网宿upos"""

    Qn = 3
    """七牛upos"""

    Kodo = 4
    """七牛bupfetch"""

    Cos = 5
    """腾讯bupfetch"""

    CosInternal = 6
    """上海腾讯云内网"""

    Bldsa = 7
    """Bldsa"""


def upload(video_path: List[str],
           cookie_file: str,
           title: str,
           tid: int,
           tag: str,
           copyright: int,
           source: str,
           desc: str,
           dynamic: str,
           cover: str,
           dolby: int,
           lossless_music: int,
           no_reprint: int,
           open_elec: int,
           limit: int,
           desc_v2: List[Credit],
           dtime: Optional[int],
           line: Optional[UploadLine]) -> None:
    """
    上传视频稿件

    :param List[str] video_path: 视频文件路径
    :param str cookie_file: cookie文件路径
    :param str title: 视频标题
    :param int tid: 投稿分区
    :param str tag: 视频标签, 英文逗号分隔多个tag
    :param int copyright: 是否转载, 1-自制 2-转载
    :param str source: 转载来源
    :param str desc: 视频简介
    :param str dynamic: 空间动态
    :param str cover: 视频封面
    :param int dolby: 是否开启杜比音效, 0-关闭 1-开启
    :param int lossless_music: 是否开启Hi-Res, 0-关闭 1-开启
    :param int no_reprint: 是否禁止转载, 0-允许 1-禁止
    :param int open_elec: 是否开启充电, 0-关闭 1-开启
    :param int limit: 单视频文件最大并发数
    :param List[Credit] desc_v2: 视频简介v2
    :param Optional[dtime] int dtime: 定时发布时间, 距离提交大于2小时小于15天, 格式为10位时间戳
    :param Optional[UploadLine] line: 上传线路
    """
