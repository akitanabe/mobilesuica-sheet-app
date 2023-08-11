"use client";
import React, { FC } from "react";
import AppProvider from "./provider";
import Header from "../components/Header";

type Props = {
  children: React.ReactNode;
};
const RootLayout: FC<Props> = ({ children }) => (
  <html lang="ja">
    <body>
      <AppProvider>
        <Header />
        <div id="root">{children}</div>
      </AppProvider>
    </body>
  </html>
);

export default RootLayout;
