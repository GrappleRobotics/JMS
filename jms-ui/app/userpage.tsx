"use client";
import { AppBar, BottomNavigation, BottomNavigationAction, Button, Container, Toolbar, Typography } from "@mui/material";
import { WebsocketComponent } from "./support/ws-component";
import React from "react";

interface UserPageProps {
  container?: boolean,
  children: React.ReactNode
};

export default class UserPage extends WebsocketComponent<UserPageProps> {
  render() {
    return <React.Fragment>
      <AppBar position="static" color={ this.isConnected() ? "default" : "error" as any } enableColorOnDark>
        <Toolbar variant="dense">
          <Button className="estop" variant="contained" sx={{ mx: 1 }}>E-STOP</Button>
          <Typography variant="h6" component="div" sx={{ml: 2}} fontWeight={700}> JMS </Typography>
          <Typography variant="h6" fontWeight={300}>&nbsp;//&nbsp;</Typography>
          { !this.isConnected() && <Typography variant="h6" fontWeight={500}> [DISCONNECTED] &nbsp;</Typography> }
        </Toolbar>
      </AppBar>
      {
        this.props.container ? <Container> { this.props.children } </Container> : this.props.children
      }
    </React.Fragment>
  }
}