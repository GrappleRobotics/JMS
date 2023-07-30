import Button from "@mui/material/Button";
import UserPage from "./userpage";

export default function Home() {
  return (
    <UserPage container>
      <h3> Hello World </h3>
      <Button variant="contained" color="purple">Hello!</Button>
    </UserPage>
  )
}
