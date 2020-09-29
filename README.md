
# daofactory
1.使用说明
所有需要安装的合约，包括基础合约（acl_app）,扩展合约（voting_app, token_app），自定义合约（根据生产需求自定义编写的合约），都需要指定APP_NAME和APP_CODE_HASH，并且在AllContracts里面定义各个合约内部初始化时需要设置的权限（权限结构体为AllPermission），结构体内所有属性都是字符串类型，值为APP_NAME

2.合约部署步骤
a.上传合约；将所有需要安装的合约上传到链上，保留各个合约的code hash；
b.将所有合约的code hash按照 使用说明的要求在dao中定义好；
c.在模板合约（community合约）中引用dao的createdao()方法
d.将模板合约编译，上传；
e.初始化模板合约时，需要携带的参数为你想要安装的合约的初始化参数（有的合约虽然在dao中定义了，但是也可以不安装）
eg."{\"init_apps\":[{\"app_name\":\"voting_app\",\"init_args\":[\"6000000000\", \"5000000000\", \"1290600000\", \"false\"]},{\"app_name\":\"token_app\",\"init_args\":[\"token\",\"TOK\",\"2\",\"0x9BA7dc2269895DF1004Ec75D8326644295508069\",\"80000\",\"0x3F43E75Aaba2c2fD6E227C10C6E7DC125A93DE3c\",\"90000\"]}]}"
以上是安装voting_app，token_app的参数，还有其他需要安装的合约时，只需要加{\"app_name\":\"\", \"init_args\":[]}即可；
f.调用模板合约的初始化方法，即可初始化所有的合约；初始化方法会返回模板合约的地址，通过模板合约的query_app方法，即可查询所有已安装的合约的地址；