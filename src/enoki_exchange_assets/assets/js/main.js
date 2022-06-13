(function ($) {
    "use strict";

    // nice select initialize
    $(document).ready(function() {
        $('select').niceSelect();
    });


    // ===== dropdown ======
    $(".wallet,.modal-dialog").click(function () {
        $(".wallet-modal").addClass("current");
    });

    $(".wallet-modal .overlay").click(function () {
        $(".wallet-modal").removeClass("current");
    });


    // ===== pool body page changer code ======

    $(".add_liquidity").click(function () {
        $(".pool_body1").removeClass("current");
        $(".pool_body2").addClass("current");
        $(".pool_body3").removeClass("current");

        // header link changer code
        $(".connect").css("display", "none");
        $(".wallet").css("display", "inline-flex");
    });
    
    $(".supply").click(function () {
        $(".pool_body1").removeClass("current");
        $(".pool_body2").removeClass("current");
        $(".pool_body3").addClass("current");

        // header link changer code
        $(".connect").css("display", "inline-block");
        $(".wallet").css("display", "none");
    });

    $("#swap-tab, #trade-tab").click(function () {
        $(".connect").css("display", "inline-block");
        $(".wallet").css("display", "none");
    });


    // // slippage popup
    // $(".setting").click(function () {
    //     $(".slippage_popup").toggleClass("current");
    //     $(".slippage_body").toggleClass("current");
    // });
    //
    // $(".slippage_popup .overly").click(function () {
    //     $(".slippage_popup").toggleClass("current");
    //     $(".slippage_body").toggleClass("current");
    // });


    

    

})(jQuery);
