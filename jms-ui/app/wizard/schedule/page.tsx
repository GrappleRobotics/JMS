"use client"
import { withPermission } from "@/app/support/permissions";
import React from "react";

export default withPermission(["ManageSchedule"], function EventWizardSchedule() {
  return <React.Fragment>
    <h3>Manage Event Schedule</h3>
  </React.Fragment>
});