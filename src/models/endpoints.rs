pub enum endpoints {
  adminAuthRegister = "/admin/auth/register",
  adminAuthLogin = "/admin/auth/login",
  adminUserDetails = "/admin/user/details",
  adminUserPassword = "/admin/user/password",
  adminQuizList = "/admin/quiz/list",
  adminQuiz = "/admin/quiz",
  adminQuizId = "/admin/quiz/{}",
  adminQuizIdName = "/admin/quiz/{}/name",
  adminQuizIdDescription = "/admin/quiz/{}/description",
  clear = "/clear", // Not available in actual api
  adminAuthLogout = "/admin/auth/logout",
  adminQuizTrash = "/admin/quiz/trash",
  adminQuizIdRestore = "/admin/quiz/{}/restore",
  adminQuizTrashEmpty = "/admin/quiz/trash/empty",
  adminQuizIdTransfer = "/admin/quiz/{}/transfer",
  adminQuiz
}