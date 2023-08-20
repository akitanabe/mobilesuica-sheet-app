"use client";

import type { Dispatch, FormEvent, SetStateAction } from "react";
import { useState, useEffect } from "react";
import Avatar from "@mui/material/Avatar";
import Button from "@mui/material/Button";
import CssBaseline from "@mui/material/CssBaseline";
import TextField from "@mui/material/TextField";
import FormControlLabel from "@mui/material/FormControlLabel";
import Checkbox from "@mui/material/Checkbox";
import Link from "@mui/material/Link";
import Grid from "@mui/material/Grid";
import Box from "@mui/material/Box";
import LockOutlinedIcon from "@mui/icons-material/LockOutlined";
import Typography from "@mui/material/Typography";
import Container from "@mui/material/Container";
import { createTheme, ThemeProvider } from "@mui/material/styles";

import axios from "axios";

type UseStateSessionId = [string, Dispatch<SetStateAction<string>>];

const fetchCaptcha = async ([
  sessionId,
  setSessionId,
]: UseStateSessionId): Promise<string> => {
  let captchaUrl = "";

  try {
    const res = await axios.get<Blob>("/api/captcha", {
      responseType: "blob",
      headers: {
        "x-session-id": sessionId,
      },
    });

    setSessionId(res.headers["x-session-id"]);
    captchaUrl = URL.createObjectURL(res.data);
  } catch (e) {
    console.log(e);
  }

  return captchaUrl;
};

const auth = async (sessionId: string) => {
  try {
    const res = await axios.get("/api/auth", {
      headers: {
        "x-session-id": sessionId,
      },
    });
  } catch (e) {
    console.log(e);
  }
};

// TODO remove, this demo shouldn't need to reset the theme.
const defaultTheme = createTheme();

export default function LogIn() {
  const handleSubmit = (
    event: FormEvent<HTMLFormElement>,
    sessionId: string
  ) => {
    event.preventDefault();
    const data = new FormData(event.currentTarget);
    console.log({
      email: data.get("email"),
      password: data.get("password"),
    });
    auth(sessionId);
  };

  const [captcha, setCaptcha] = useState<string>("");
  const [sessionId, setSessionId] = useState<string>("");

  useEffect(() => {
    if (captcha === "") {
      fetchCaptcha([sessionId, setSessionId]).then((captcha) =>
        setCaptcha(captcha)
      );
    }
  }, []);

  return (
    <ThemeProvider theme={defaultTheme}>
      <Container component="main" maxWidth="xs">
        <CssBaseline />
        <Box
          sx={{
            marginTop: 8,
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
          }}
        >
          <Avatar sx={{ m: 1, bgcolor: "secondary.main" }}>
            <LockOutlinedIcon />
          </Avatar>
          <Typography component="h1" variant="h5">
            モバイルSuica ログイン
          </Typography>
          <Box
            component="form"
            onSubmit={(e) => handleSubmit(e, sessionId)}
            noValidate
            sx={{ mt: 1 }}
          >
            <TextField
              margin="normal"
              required
              fullWidth
              id="email"
              label="メールアドレス"
              name="email"
              autoComplete="email"
              autoFocus
            />
            <TextField
              margin="normal"
              required
              fullWidth
              name="password"
              label="パスワード"
              type="password"
              id="password"
              autoComplete="current-password"
            />
            <FormControlLabel
              control={<Checkbox value="remember" color="primary" />}
              label="Remember me"
            />
            <Box>
              <img src={captcha} />
            </Box>
            <Button
              type="submit"
              fullWidth
              variant="contained"
              sx={{ mt: 3, mb: 2 }}
            >
              Log In
            </Button>
          </Box>
        </Box>
      </Container>
    </ThemeProvider>
  );
}
