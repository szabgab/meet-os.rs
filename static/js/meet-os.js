document.addEventListener('DOMContentLoaded', () => {

  // Get all "navbar-burger" elements
  const $navbarBurgers = Array.prototype.slice.call(document.querySelectorAll('.navbar-burger'), 0);

  // Add a click event on each of them
  $navbarBurgers.forEach( el => {
    el.addEventListener('click', () => {

      // Get the target from the "data-target" attribute
      const target = el.dataset.target;
      const $target = document.getElementById(target);

      // Toggle the "is-active" class on both the "navbar-burger" and the "navbar-menu"
      el.classList.toggle('is-active');
      $target.classList.toggle('is-active');

    });
  });

  function set_local_date() {
    // Assume a date format of "2021-04-13T19:00:00+03:00";
    // Display time in localtime of the browser.

    const dates = document.getElementsByClassName("datetime");
    for (let ix=0; ix < dates.length; ix++) {
        //const mydate = dates[ix].getAttribute("x-schedule");
        const mydate = dates[ix].getAttribute("value");
        const date = new Date(mydate);

        dates[ix].innerHTML = date.toLocaleDateString( [], {
            weekday: 'long',
            year: 'numeric',
            month: 'long',
            day: 'numeric',
            hour: 'numeric',
            minute: 'numeric',
            timeZoneName: 'long'
        });
    }
  }

  function set_local_timezone() {
    if (document.getElementById("edit-event")) {
      const date = new Date();
      //console.log(Intl.DateTimeFormat().resolvedOptions().timeZone); // e.g. Asia/Jerusalem
      //console.log(date.toLocaleDateString(undefined, {day:'2-digit',timeZoneName: 'long' }).substring(4)); // Israel Daylight Time
      //console.log(date.toLocaleDateString(undefined, {day:'2-digit',timeZoneName: 'short' }).substring(4)); // GMT+3
      //const offset = date.getTimezoneOffset();
      //console.log(offset); // -180
      let text = date.toLocaleDateString(undefined, {day:'2-digit',timeZoneName: 'short' }).substring(4);
      text += "(" + date.toLocaleDateString(undefined, {day:'2-digit',timeZoneName: 'long' }).substring(4) + ")";
      document.getElementById("timezone").innerHTML = text;
      document.getElementById("offset").value = date.getTimezoneOffset();
    }
  }

  set_local_timezone();
  set_local_date();
});
