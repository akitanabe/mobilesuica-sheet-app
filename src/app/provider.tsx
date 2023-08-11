import { FC, ReactNode } from "react";
import { RecoilRoot } from "recoil";

type Props = { children: ReactNode };

const AppProvider: FC<Props> = ({ children }: Props) => {
  return <RecoilRoot>{children}</RecoilRoot>;
};

export default AppProvider;
